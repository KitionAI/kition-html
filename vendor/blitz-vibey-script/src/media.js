// Media evaluation is shared with Stylo; a preview session has a fixed viewport.
(() => {
    const evaluate = __blitz_match_media;
    class MediaQueryList {
        #query;
        #target = document.createDocumentFragment();
        constructor(query) { this.#query = evaluate(String(query))[0]; }
        get media() { return this.#query; }
        get matches() { return evaluate(this.#query)[1]; }
        onchange = null;
        addEventListener(...args) { this.#target.addEventListener(...args); }
        removeEventListener(...args) { this.#target.removeEventListener(...args); }
        addListener(callback) { this.addEventListener("change", callback); }
        removeListener(callback) { this.removeEventListener("change", callback); }
        dispatchEvent(event) {
            const result = this.#target.dispatchEvent(event);
            if (event.type === "change" && typeof this.onchange === "function") this.onchange(event);
            return result;
        }
    }
    globalThis.MediaQueryList = MediaQueryList;
    globalThis.matchMedia = query => new MediaQueryList(query);
})();
