// Live collections resolve membership on every read, retaining stable node wrappers.
(() => {
    const data = new WeakMap();
    const entries = list => {
        const read = data.get(list);
        if (!read) throw new TypeError("Invalid collection receiver");
        return read();
    };
    const index = property => typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property) && Number(property) < 4294967295;
    class HTMLCollection {
        constructor() { throw new TypeError("Illegal constructor"); }
        get length() { return entries(this).length; }
        item(index) { return entries(this)[Number(index) >>> 0] ?? null; }
        namedItem(name) {
            name = String(name);
            if (!name) return null;
            return entries(this).find(node => node.id === name || node.getAttribute("name") === name) ?? null;
        }
        *[Symbol.iterator]() { for (let i=0; i<this.length; i++) yield this.item(i); }
    }
    class NodeList {
        constructor() { throw new TypeError("Illegal constructor"); }
        get length() { return entries(this).length; }
        item(index) { return entries(this)[Number(index) >>> 0] ?? null; }
        *values() { for (let i=0; i<this.length; i++) yield this.item(i); }
        *keys() { for (let i=0; i<this.length; i++) yield i; }
        *entries() { for (let i=0; i<this.length; i++) yield [i,this.item(i)]; }
        forEach(callback, thisArg) {
            const length=this.length;
            for(let i=0;i<length;i++) { const value=this[i]; if(value!==undefined) callback.call(thisArg,value,i,this); }
        }
        [Symbol.iterator]() { return this.values(); }
    }
    function collection(Type, read) {
        const target=Object.create(Type.prototype);
        const proxy=new Proxy(target, {
            get(target, property, receiver) {
                if(index(property)) return read()[Number(property)];
                if(Type===HTMLCollection && typeof property==='string' && !(property in target)) return receiver.namedItem(property) ?? undefined;
                return Reflect.get(target,property,receiver);
            },
            has(target, property) {
                return index(property) ? Number(property)<read().length : Reflect.has(target,property) || (Type===HTMLCollection && typeof property==='string' && proxy.namedItem(property)!==null);
            },
            ownKeys(target) { return [...read().map((_,i)=>String(i)),...Reflect.ownKeys(target)]; },
            getOwnPropertyDescriptor(target,property) {
                if(index(property)) {
                    const value=read()[Number(property)];
                    return value===undefined ? undefined : {value,configurable:true,enumerable:true,writable:false};
                }
                return Reflect.getOwnPropertyDescriptor(target,property);
            },
            set(target,property,value,receiver) {
                if(index(property)||property==='length') return false;
                return Reflect.set(target,property,value,receiver);
            },
        });
        data.set(proxy,read);
        return proxy;
    }
    for(const proto of [Document.prototype, Element.prototype]) {
        for(const name of ['getElementsByClassName','getElementsByTagName']) {
            const native=proto[name];
            proto[name]=function(query) {
                query=String(query);
                native.call(this,query);
                return collection(HTMLCollection,()=>native.call(this,query));
            };
        }
        const query=proto.querySelectorAll;
        proto.querySelectorAll=function(selector) {
            const result=query.call(this,selector);
            return collection(NodeList,()=>result);
        };
    }
    for(const [proto,name,Type] of [[Node.prototype,'childNodes',NodeList],[Element.prototype,'children',HTMLCollection]]) {
        const descriptor=Object.getOwnPropertyDescriptor(proto,name);
        if(!descriptor?.get) continue;
        const cached=new WeakMap();
        Object.defineProperty(proto,name,{configurable:true,enumerable:descriptor.enumerable,get() {
            if(!cached.has(this)) cached.set(this,collection(Type,()=>descriptor.get.call(this)));
            return cached.get(this);
        }});
    }
    Object.assign(globalThis,{HTMLCollection,NodeList});
})();
