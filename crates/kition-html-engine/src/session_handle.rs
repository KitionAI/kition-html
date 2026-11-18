//! Keep Boa on one thread while exposing a Send handle to the native kit server.
use crate::{RenderOptions, Rendered, Resources, Session};

#[cfg(target_arch = "wasm32")]
pub struct SessionHandle(std::cell::RefCell<Session>);

#[cfg(not(target_arch = "wasm32"))]
type Work = Box<dyn FnOnce(&mut Session) + Send>;
#[cfg(not(target_arch = "wasm32"))]
pub struct SessionHandle(std::sync::mpsc::Sender<Work>);

impl SessionHandle {
    pub fn new(html: String, resources: Resources, options: RenderOptions) -> Result<Self, String> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Self(std::cell::RefCell::new(Session::new(
                &html, resources, &options,
            )?)))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (send, receive) = std::sync::mpsc::channel::<Work>();
            let (ready, result) = std::sync::mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("html-script".into())
                .stack_size(8 * 1024 * 1024)
                .spawn(move || match Session::new(&html, resources, &options) {
                    Ok(mut session) => {
                        if ready.send(Ok(())).is_err() {
                            return;
                        }
                        for work in receive {
                            work(&mut session);
                        }
                    }
                    Err(error) => {
                        let _ = ready.send(Err(error));
                    }
                })
                .map_err(|error| error.to_string())?;
            result
                .recv()
                .map_err(|_| "script thread stopped during startup")??;
            Ok(Self(send))
        }
    }

    fn call<T: Send + 'static>(
        &self,
        call: impl FnOnce(&mut Session) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        #[cfg(target_arch = "wasm32")]
        {
            call(&mut self.0.borrow_mut())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (send, receive) = std::sync::mpsc::sync_channel(1);
            self.0
                .send(Box::new(move |session| {
                    let _ = send.send(call(session));
                }))
                .map_err(|_| "script thread is unavailable")?;
            receive.recv().map_err(|_| "script thread stopped")?
        }
    }

    pub fn frame(&self, scale: f64) -> Result<Rendered, String> {
        self.call(move |session| session.frame(scale))
    }
    pub fn advance(&self, millis: u64) -> Result<bool, String> {
        self.call(move |session| session.advance(millis))
    }
    pub fn click(&self, x: f32, y: f32) -> Result<bool, String> {
        self.call(move |session| session.click(x, y))
    }
}
