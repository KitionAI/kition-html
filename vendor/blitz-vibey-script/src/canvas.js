// Canvas state and WebIDL-facing validation. Pixels are produced by the CPU engine.
(() => {
    const native = __blitz_canvas;
    const contexts = new WeakMap(), states = new WeakMap(), gradients = new WeakMap(), paths = new WeakMap(), fontStyles = new WeakMap();
    const matrix = () => [1,0,0,1,0,0];
    const initial = () => ({transform:matrix(),fillStyle:'#000000',strokeStyle:'#000000',globalAlpha:1,globalCompositeOperation:'source-over',lineWidth:1,lineCap:'butt',lineJoin:'miter',miterLimit:10,lineDashOffset:0,dash:[],clips:[],imageSmoothingEnabled:true,font:'10px sans-serif',fontData:{size:10,family:'sans-serif',weight:400,style:'normal'},letterSpacing:'0px',wordSpacing:'0px',letterSpacingData:{value:'0',unit:'px'},wordSpacingData:{value:'0',unit:'px'},fontKerning:'auto',textAlign:'start',textBaseline:'alphabetic',direction:'inherit'});
    const call = (canvas, value) => JSON.parse(native(canvas, JSON.stringify(value)));
    function state(context) {
        const data=states.get(context);
        if(!data) throw new TypeError('Invalid canvas context receiver');
        const info=call(data.canvas,{op:'info'});
        if(info.revision!==data.revision) {
            data.revision=info.revision;data.state=initial();data.stack=[];data.path=[];
        }
        return data;
    }
    const numbers = args => args.map(Number);
    const finite = args => args.every(Number.isFinite);
    const multiply = (a,b) => [a[0]*b[0]+a[2]*b[1],a[1]*b[0]+a[3]*b[1],a[0]*b[2]+a[2]*b[3],a[1]*b[2]+a[3]*b[3],a[0]*b[4]+a[2]*b[5]+a[4],a[1]*b[4]+a[3]*b[5]+a[5]];
    function color(canvas,value) {return call(canvas,{op:'color',value:String(value)==='currentColor'?getComputedStyle(canvas).color:String(value)});}
    function brush(value) {const data=gradients.get(value);if(!data)return value;const {canvas,...paint}=data;return paint;}
    function path(context,op,args) {
        const data=paths.get(context)??state(context);args=numbers(args);if(!finite(args))return;
        if(data.path.length>=16384)throw new RangeError('Canvas path command limit exceeded');
        data.path.push([op,args,data.state.transform.slice()]);
    }
    function commands(data,source) {
        if(source===undefined)return data.path;
        if(!paths.has(source))throw new TypeError('Expected Path2D');
        return paths.get(source).path.map(([op,args,transform])=>[op,args,multiply(data.state.transform,transform)]);
    }
    function paint(context,op,args=[],rule='nonzero',source) {
        const data=state(context);args=numbers(args);if(!finite(args))return;
        if(!['nonzero','evenodd'].includes(rule))throw new TypeError('Invalid canvas fill rule');
        call(data.canvas,{op,args,path:commands(data,source),rule,state:{...data.state,fillStyle:brush(data.state.fillStyle),strokeStyle:brush(data.state.strokeStyle)}});
    }
    function numericArguments(context,args,count,required) {
        state(context);if(count<required)throw new TypeError('Not enough Canvas numeric arguments');
        args=args.map(value=>+value);return finite(args)?args:null;
    }
    function transformBy(context,args) {
        const d=state(context);d.state.transform=multiply(d.state.transform,args);
    }
    function text(context,op,value,args=[],count=1) {
        state(context);if(count<(op==='measureText'?1:3))throw new TypeError('Not enough Canvas text arguments');
        if(typeof value==='symbol')throw new TypeError('Cannot convert a Symbol to Canvas text');
        value=String(value);args=args.map(value=>+value);
        if(!finite(args)||(args.length===3&&args[2]<=0))return;
        if(value.length>65536)throw new RangeError('Canvas text input limit exceeded');
        const d=state(context);
        const direction=d.state.direction==='inherit'?(getComputedStyle(d.canvas).direction||'ltr'):d.state.direction;
        return call(d.canvas,{op,text:value,args,state:{...d.state,direction,fillStyle:brush(d.state.fillStyle),strokeStyle:brush(d.state.strokeStyle)}});
    }
    class Path2D {
        constructor(source) {
            let path=[];
            if(paths.has(source))path=paths.get(source).path.slice();
            else if(source!==undefined){const text=String(source);if(text.length>65536)throw new RangeError('SVG path input limit exceeded');path=[['svg',[text],matrix()]];}
            paths.set(this,{path,state:{transform:matrix()}});
        }
        addPath(source,transform={}) {
            const data=paths.get(this),other=paths.get(source);if(!data||!other)throw new TypeError('Expected Path2D');
            const m=DOMMatrix.fromMatrix(transform),values=[m.a,m.b,m.c,m.d,m.e,m.f];if(!finite(values))return;
            if(data.path.length+other.path.length>16384)throw new RangeError('Canvas path command limit exceeded');
            data.path.push(...other.path.map(([op,args,previous])=>[op,args,multiply(values,previous)]));
        }
    }
    class TextMetrics {
        constructor(token,values){if(token!==states)throw new TypeError('Illegal constructor');for(const [key,value] of Object.entries(values))Object.defineProperty(this,key,{value,enumerable:true});}
    }
    class CanvasGradient {
        constructor(token,data) {if(token!==gradients)throw new TypeError('Illegal constructor');gradients.set(this,data);}
        addColorStop(offset,value) {
            offset=Number(offset);
            if(!Number.isFinite(offset)||offset<0||offset>1)throw new DOMException('Color stop is out of range','IndexSizeError');
            const data=gradients.get(this);if(!data)throw new TypeError('Invalid gradient receiver');
            const parsed=color(data.canvas,value);if(parsed===null)throw new DOMException('Invalid color','SyntaxError');
            data.stops.push([offset,parsed]);data.stops.sort((a,b)=>a[0]-b[0]);
        }
    }
    class ImageData {
        constructor(data,width,height) {
            if(typeof data==='number') {
                height=Number(width)>>>0;width=Number(data)>>>0;
                if(!width||!height||width*height>16777216)throw new DOMException('Invalid image dimensions','IndexSizeError');
                data=new Uint8ClampedArray(width*height*4);
            } else {
                if(!(data instanceof Uint8ClampedArray))throw new TypeError('Expected Uint8ClampedArray');
                width=Number(width)>>>0;height=height===undefined?data.length/(4*width):Number(height)>>>0;
                if(!width||!height||!Number.isInteger(height)||data.length!==width*height*4)throw new DOMException('Invalid image data length','IndexSizeError');
            }
            Object.defineProperties(this,{data:{value:data,enumerable:true},width:{value:width,enumerable:true},height:{value:height,enumerable:true},colorSpace:{value:'srgb',enumerable:true}});
        }
    }
    class CanvasRenderingContext2D {
        constructor(token,canvas) {
            if(token!==contexts)throw new TypeError('Illegal constructor');
            states.set(this,{canvas,revision:call(canvas,{op:'info'}).revision,state:initial(),stack:[],path:[]});
        }
        get canvas(){return state(this).canvas;}
        save(){const d=state(this);if(d.stack.length>=1024)throw new RangeError('Canvas state stack limit exceeded');d.stack.push({...d.state,transform:d.state.transform.slice(),dash:d.state.dash.slice(),clips:d.state.clips.slice()});}
        restore(){const d=state(this);if(d.stack.length)d.state=d.stack.pop();}
        reset(){const d=state(this);d.canvas.width=d.canvas.width;state(this);}
        beginPath(){state(this).path=[];}
        closePath(){path(this,'close',[]);}
        moveTo(x,y){path(this,'move',[x,y]);}
        lineTo(x,y){path(this,'line',[x,y]);}
        rect(x,y,w,h){path(this,'rect',[x,y,w,h]);}
        roundRect(x,y,w,h,radii=0){
            const rect=numbers([x,y,w,h]);if(!finite(rect))return;
            const values=typeof radii==='object'&&radii!==null&&Symbol.iterator in radii?Array.from(radii):[radii];
            if(values.length<1||values.length>4)throw new RangeError('Expected one to four corner radii');
            const corners=values.map(value=>typeof value==='object'&&value!==null?[Number(value.x??0),Number(value.y??0)]:[Number(value),Number(value)]);
            if(corners.some(pair=>pair.some(value=>value<0)))throw new RangeError('Negative corner radius');
            if(!corners.every(finite))return;
            const [a,b=a,c=a,d=b]=corners;path(this,'roundRect',[...rect,...a,...b,...c,...d]);
        }
        quadraticCurveTo(cx,cy,x,y){path(this,'quadratic',[cx,cy,x,y]);}
        bezierCurveTo(a,b,c,d,x,y){path(this,'bezier',[a,b,c,d,x,y]);}
        arc(x,y,r,start,end,ccw=false){this.ellipse(x,y,r,r,0,start,end,ccw);}
        arcTo(x1,y1,x2,y2,radius){
            const args=numbers([x1,y1,x2,y2,radius]);if(!finite(args))return;
            if(args[4]<0)throw new DOMException('Negative arc radius','IndexSizeError');
            path(this,'arcTo',args);
        }
        ellipse(x,y,rx,ry,rotation,start,end,ccw=false){
            const args=numbers([x,y,rx,ry,rotation,start,end]);if(!finite(args))return;
            if(args[2]<0||args[3]<0)throw new DOMException('Negative ellipse radius','IndexSizeError');
            path(this,'ellipse',[...args,ccw?1:0]);
        }
        fill(source='nonzero',rule='nonzero'){if(paths.has(source))paint(this,'fill',[],rule,source);else paint(this,'fill',[],source);}
        stroke(source){paint(this,'stroke',[],'nonzero',source);}
        clip(source='nonzero',rule='nonzero'){
            if(!paths.has(source)){rule=source;source=undefined;}
            const d=state(this);if(!['nonzero','evenodd'].includes(rule))throw new TypeError('Invalid canvas fill rule');
            if(d.state.clips.length>=128)throw new RangeError('Canvas clip limit exceeded');
            d.state.clips.push({path:commands(d,source).slice(),rule});
        }
        isPointInPath(...args){return this._hit('hitFill',args);}
        isPointInStroke(...args){return this._hit('hitStroke',args);}
        _hit(op,args){
            const d=state(this),source=paths.has(args[0])?args.shift():undefined;
            const coords=numbers(args.slice(0,2)),rule=op==='hitFill'?(args[2]??'nonzero'):'nonzero';
            if(!['nonzero','evenodd'].includes(rule))throw new TypeError('Invalid canvas fill rule');
            if(coords.length!==2||!finite(coords))return false;
            return call(d.canvas,{op,args:coords,path:commands(d,source),rule,state:d.state});
        }
        fillRect(x,y,w,h){const args=numericArguments(this,[x,y,w,h],arguments.length,4);if(args)paint(this,'fillRect',args);}
        strokeRect(x,y,w,h){const args=numericArguments(this,[x,y,w,h],arguments.length,4);if(args)paint(this,'strokeRect',args);}
        clearRect(x,y,w,h){const args=numericArguments(this,[x,y,w,h],arguments.length,4);if(args)paint(this,'clearRect',args);}
        drawImage(image,...args){
            const d=state(this);
            if(![2,4,8].includes(args.length))throw new TypeError('Invalid drawImage argument count');
            if(!image||!['CANVAS','IMG'].includes(image.tagName))throw new TypeError('Expected an image or canvas');
            args=numbers(args);if(!finite(args))return;
            if(image.tagName==='CANVAS'&&(!image.width||!image.height))throw new DOMException('Empty source canvas','InvalidStateError');
            native(d.canvas,JSON.stringify({op:'drawImage',args,state:{...d.state,fillStyle:brush(d.state.fillStyle),strokeStyle:brush(d.state.strokeStyle)}}),image);
        }
        fillText(value,x,y,maxWidth=undefined){text(this,'fillText',value,maxWidth===undefined?[x,y]:[x,y,maxWidth],arguments.length);}
        strokeText(value,x,y,maxWidth=undefined){text(this,'strokeText',value,maxWidth===undefined?[x,y]:[x,y,maxWidth],arguments.length);}
        measureText(value){return new TextMetrics(states,text(this,'measureText',value,[],arguments.length));}
        translate(x,y){const args=numericArguments(this,[x,y],arguments.length,2);if(args)transformBy(this,[1,0,0,1,...args]);}
        scale(x,y){const args=numericArguments(this,[x,y],arguments.length,2);if(args)transformBy(this,[args[0],0,0,args[1],0,0]);}
        rotate(angle){const args=numericArguments(this,[angle],arguments.length,1);if(args){const c=Math.cos(args[0]),s=Math.sin(args[0]);transformBy(this,[c,s,-s,c,0,0]);}}
        transform(a,b,c,d,e,f){const args=numericArguments(this,[a,b,c,d,e,f],arguments.length,6);if(args)transformBy(this,args);}
        setTransform(...args){
            if(args.length===0)args=matrix();else if(args.length===1){const m=DOMMatrix.fromMatrix(args[0]);args=[m.a,m.b,m.c,m.d,m.e,m.f];}
            args=numbers(args);if(args.length===6&&finite(args))state(this).state.transform=args;
        }
        resetTransform(){state(this).state.transform=matrix();}
        getTransform(){return new DOMMatrix(state(this).state.transform);}
        setLineDash(values){
            state(this);
            const object=value=>value!==null&&(typeof value==='object'||typeof value==='function');
            if(!object(values))throw new TypeError('Expected an iterable dash sequence');
            const method=values[Symbol.iterator];
            if(typeof method!=='function')throw new TypeError('Expected an iterable dash sequence');
            const iterator=Reflect.apply(method,values,[]);
            if(!object(iterator))throw new TypeError('Expected an iterator object');
            const next=iterator.next,dash=[];
            // WebIDL sequences convert each value before advancing and do not close on failure.
            while(true){
                const result=Reflect.apply(next,iterator,[]);
                if(!object(result))throw new TypeError('Expected an iterator result');
                if(result.done)break;
                dash.push(+result.value);
            }
            if(dash.some(x=>x<0)||!finite(dash))return;
            state(this).state.dash=dash.length%2?[...dash,...dash]:dash;
        }
        getLineDash(){return state(this).state.dash.slice();}
        createLinearGradient(...args){return this._gradient('linear',args);}
        createRadialGradient(...args){return this._gradient('radial',args);}
        _gradient(kind,args){
            args=numbers(args);if(!finite(args))throw new TypeError('Non-finite gradient coordinate');
            if(kind==='radial'&&(args[2]<0||args[5]<0))throw new DOMException('Negative gradient radius','IndexSizeError');
            const d=state(this);return new CanvasGradient(gradients,{kind,args,transform:d.state.transform.slice(),stops:[],canvas:d.canvas});
        }
        createImageData(width,height){if(width instanceof ImageData)return new ImageData(width.width,width.height);return new ImageData(Math.abs(Math.trunc(Number(width))),Math.abs(Math.trunc(Number(height))));}
        getImageData(x,y,width,height){
            const d=state(this),args=numbers([x,y,width,height]).map(Math.trunc);
            if(!finite(args))throw new TypeError('Non-finite pixel rectangle');
            if(!args[2]||!args[3])throw new DOMException('Empty pixel rectangle','IndexSizeError');
            const result=call(d.canvas,{op:'read',args});return new ImageData(new Uint8ClampedArray(result.data),result.width,result.height);
        }
        putImageData(image,x,y,...dirty){
            if(!(image instanceof ImageData))throw new TypeError('Expected ImageData');
            const args=numbers([x,y,...dirty]).map(Math.trunc);if(!finite(args))throw new TypeError('Non-finite pixel rectangle');
            call(state(this).canvas,{op:'write',args,width:image.width,height:image.height,data:Array.from(image.data)});
        }
        getContextAttributes(){return {alpha:true,colorSpace:'srgb',desynchronized:false,willReadFrequently:false};}
        isContextLost(){return false;}
    }
    for(const key of ['fillStyle','strokeStyle'])Object.defineProperty(CanvasRenderingContext2D.prototype,key,{get(){return state(this).state[key]},set(value){const d=state(this);if(gradients.has(value)){d.state[key]=value;return;}const parsed=color(d.canvas,value);if(parsed!==null)d.state[key]=parsed;},configurable:true});
    for(const key of ['globalAlpha','lineWidth','miterLimit','lineDashOffset'])Object.defineProperty(CanvasRenderingContext2D.prototype,key,{get(){return state(this).state[key]},set(value){state(this);value=+value;if(!Number.isFinite(value)||(key==='globalAlpha'&&(value<0||value>1))||(['lineWidth','miterLimit'].includes(key)&&value<=0))return;state(this).state[key]=value;},configurable:true});
    Object.defineProperty(CanvasRenderingContext2D.prototype,'font',{get(){return state(this).state.font},set(value){
        state(this);if(typeof value==='symbol')throw new TypeError('Cannot convert a Symbol to a font');value=String(value);
        const d=state(this);if(!fontStyles.has(this))fontStyles.set(this,document.createElement('span').style);
        const style=fontStyles.get(this);style.cssText='';style.font=value;if(!style.font||!style.fontFamily)return;
        const match=/^([\d.]+)(px|pt|pc|in|cm|mm|q|em|rem|%)$/.exec(style.fontSize);if(!match)return;
        const inherited=parseFloat(getComputedStyle(d.canvas).fontSize)||16;
        const root=parseFloat(getComputedStyle(document.documentElement).fontSize)||16;
        const units={px:1,pt:96/72,pc:16,in:96,cm:96/2.54,mm:96/25.4,q:96/101.6,em:inherited,rem:root,'%':inherited/100};
        const size=Number(match[1])*units[match[2]];if(!Number.isFinite(size)||size<=0)return;
        d.state.font=style.font;d.state.fontData={size,family:style.fontFamily,weight:style.fontWeight==='bold'?700:Number(style.fontWeight)||400,style:style.fontStyle||'normal'};
    },configurable:true});
    for(const property of ['letterSpacing','wordSpacing'])Object.defineProperty(CanvasRenderingContext2D.prototype,property,{
        get(){return state(this).state[property]},
        set(value){
            state(this);if(typeof value==='symbol')throw new TypeError('Cannot convert a Symbol to a spacing length');value=String(value);
            const d=state(this),parsed=call(d.canvas,{op:'spacing',value});if(parsed===null)return;
            let number=parsed.value.toPrecision(6);if(!number.includes('e'))number=String(Number(number));
            d.state[property]=number+parsed.unit;d.state[property+'Data']={value:String(parsed.value),unit:parsed.unit};
        },enumerable:true,configurable:true,
    });
    Object.defineProperty(CanvasRenderingContext2D.prototype,'fontKerning',{
        get(){return state(this).state.fontKerning},
        set(value){state(this);if(typeof value==='symbol')throw new TypeError('Cannot convert a Symbol to a kerning mode');value=String(value);if(['auto','normal','none'].includes(value))state(this).state.fontKerning=value;},
        enumerable:true,configurable:true,
    });
    const enums={textAlign:['start','end','left','right','center'],textBaseline:['top','hanging','middle','alphabetic','ideographic','bottom'],direction:['inherit','ltr','rtl'],lineCap:['butt','round','square'],lineJoin:['miter','round','bevel'],globalCompositeOperation:['source-over','source-in','source-out','source-atop','destination-over','destination-in','destination-out','destination-atop','lighter','copy','xor','multiply','screen','overlay','darken','lighten','difference','exclusion']};
    for(const [key,values] of Object.entries(enums))Object.defineProperty(CanvasRenderingContext2D.prototype,key,{get(){return state(this).state[key]},set(value){value=String(value);if(values.includes(value))state(this).state[key]=value;},configurable:true});
    Object.defineProperty(CanvasRenderingContext2D.prototype,'imageSmoothingEnabled',{get(){return state(this).state.imageSmoothingEnabled},set(value){state(this).state.imageSmoothingEnabled=Boolean(value)},configurable:true});
    HTMLElement.prototype.getContext=function(type){
        if(this.tagName!=='CANVAS')throw new TypeError('getContext requires a canvas');
        if(String(type)!=='2d')return null;
        if(!contexts.has(this))contexts.set(this,new CanvasRenderingContext2D(contexts,this));
        return contexts.get(this);
    };
    for(const method of ['closePath','moveTo','lineTo','rect','roundRect','quadraticCurveTo','bezierCurveTo','arc','arcTo','ellipse'])Object.defineProperty(Path2D.prototype,method,{value:CanvasRenderingContext2D.prototype[method],writable:true,configurable:true});
    Object.assign(globalThis,{CanvasRenderingContext2D,CanvasGradient,ImageData,Path2D,TextMetrics});
})();
