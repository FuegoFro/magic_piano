const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : (typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : undefined));
let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_32(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h9c293761ae856fa9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_35(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha5e546b7a5c9454a(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_38(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h01c7a18edec73b1c(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_41(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h149df7d1a0f792f5(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_44(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h69fcb478986cdff7(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_47(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hced6384173e0bdb9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_50(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he250af3924df3ca0(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_53(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd977d08548b2160d(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_56(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h8e2f20888cfa6179(arg0, arg1, addHeapObject(arg2));
}

function makeClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_59(arg0, arg1) {
    wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbb36a19ecd5cfb99(arg0, arg1);
}

function __wbg_adapter_62(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h92400e7785e840dc(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_65(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1527e32ceda3957e(arg0, arg1);
}

function __wbg_adapter_68(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h75a8dc68b5288a6c(arg0, arg1, addHeapObject(arg2));
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getUint32Memory0();
    for (let i = 0; i < array.length; i++) {
        mem[ptr / 4 + i] = addHeapObject(array[i]);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(takeObject(slice[i]));
    }
    return result;
}

function getCachedStringFromWasm0(ptr, len) {
    if (ptr === 0) {
        return getObject(len);
    } else {
        return getStringFromWasm0(ptr, len);
    }
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

let cachedUint16Memory0 = null;

function getUint16Memory0() {
    if (cachedUint16Memory0 === null || cachedUint16Memory0.byteLength === 0) {
        cachedUint16Memory0 = new Uint16Array(wasm.memory.buffer);
    }
    return cachedUint16Memory0;
}

function getArrayU16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint16Memory0().subarray(ptr / 2, ptr / 2 + len);
}
function __wbg_adapter_485(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h913d476d70146797(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

/**
*/
export const StemDirectionType = Object.freeze({ Up:0,"0":"Up",Down:1,"1":"Down",None:2,"2":"None",Double:3,"3":"Double", });

const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr >>> 0));
/**
*/
export class IntoUnderlyingByteSource {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingByteSourceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr);
    }
    /**
    * @returns {string}
    */
    get type() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.intounderlyingbytesource_type(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getCachedStringFromWasm0(r0, r1);
        if (r0 !== 0) { wasm.__wbindgen_free(r0, r1, 1); }
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
/**
* @returns {number}
*/
get autoAllocateChunkSize() {
    const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
    return ret >>> 0;
}
/**
* @param {ReadableByteStreamController} controller
*/
start(controller) {
    wasm.intounderlyingbytesource_start(this.__wbg_ptr, addHeapObject(controller));
}
/**
* @param {ReadableByteStreamController} controller
* @returns {Promise<any>}
*/
pull(controller) {
    const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, addHeapObject(controller));
    return takeObject(ret);
}
/**
*/
cancel() {
    const ptr = this.__destroy_into_raw();
    wasm.intounderlyingbytesource_cancel(ptr);
}
}

const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr >>> 0));
/**
*/
export class IntoUnderlyingSink {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSinkFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr);
    }
    /**
    * @param {any} chunk
    * @returns {Promise<any>}
    */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, addHeapObject(chunk));
        return takeObject(ret);
    }
    /**
    * @returns {Promise<any>}
    */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return takeObject(ret);
    }
    /**
    * @param {any} reason
    * @returns {Promise<any>}
    */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, addHeapObject(reason));
        return takeObject(ret);
    }
}

const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr >>> 0));
/**
*/
export class IntoUnderlyingSource {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSourceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr);
    }
    /**
    * @param {ReadableStreamDefaultController} controller
    * @returns {Promise<any>}
    */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, addHeapObject(controller));
        return takeObject(ret);
    }
    /**
    */
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_is_null = function(arg0) {
        const ret = getObject(arg0) === null;
        return ret;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = getObject(arg0) === getObject(arg1);
        return ret;
    };
    imports.wbg.__wbg_instanceof_OpenSheetMusicDisplay_6a6f4f61924162d7 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof opensheetmusicdisplay.OpenSheetMusicDisplay;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_new_c34739d83dd10565 = function(arg0) {
        const ret = new opensheetmusicdisplay.OpenSheetMusicDisplay(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_setzoom_bb2c49fc2b7cd303 = function(arg0, arg1) {
        getObject(arg0).zoom = arg1;
    };
    imports.wbg.__wbg_load_6e1153682b5f3cd6 = function(arg0, arg1) {
        const ret = getObject(arg0).load(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_render_3b135fd5e31aafea = function(arg0) {
        getObject(arg0).render();
    };
    imports.wbg.__wbg_cursor_8737b7bd0bad4e58 = function(arg0) {
        const ret = getObject(arg0).cursor;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_cursors_f029134928dcc30a = function(arg0, arg1) {
        const ret = getObject(arg1).cursors;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_setcursorsOptions_7b8e6c8a11fc809d = function(arg0, arg1, arg2) {
        var v0 = getArrayJsValueFromWasm0(arg1, arg2).slice();
        wasm.__wbindgen_free(arg1, arg2 * 4, 4);
        getObject(arg0).cursorsOptions = v0;
    };
    imports.wbg.__wbg_sheet_4da53092cd1cf8c6 = function(arg0) {
        const ret = getObject(arg0).sheet;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_graphic_03c8efca6ce9a6ad = function(arg0) {
        const ret = getObject(arg0).graphic;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_handleResize_381d10502b68005c = function(arg0, arg1, arg2) {
        getObject(arg0).handleResize(getObject(arg1), getObject(arg2));
    };
    imports.wbg.__wbg_rules_c686bda9fbca1640 = function(arg0) {
        const ret = getObject(arg0).rules;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_staves_ef67b716ff822d49 = function(arg0, arg1) {
        const ret = getObject(arg1).staves;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_parentStaff_4a830cee16b8c885 = function(arg0) {
        const ret = getObject(arg0).parentStaff;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_idInMusicSheet_def09eca3e3871b2 = function(arg0) {
        const ret = getObject(arg0).idInMusicSheet;
        return ret;
    };
    imports.wbg.__wbg_voices_532e3ee98a35f7e0 = function(arg0, arg1) {
        const ret = getObject(arg1).voices;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_absoluteTimestamp_b187e8aa265d863d = function(arg0) {
        const ret = getObject(arg0).absoluteTimestamp;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_verticalGraphicalStaffEntryContainers_657be475ba35a9b6 = function(arg0, arg1) {
        const ret = getObject(arg1).verticalGraphicalStaffEntryContainers;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_musicPages_78d8500e4865975a = function(arg0, arg1) {
        const ret = getObject(arg1).musicPages;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_staffEntries_e372b07973ae09fd = function(arg0, arg1) {
        const ret = getObject(arg1).staffEntries;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_PositionAndShape_14b175e1b4309e2a = function(arg0) {
        const ret = getObject(arg0).PositionAndShape;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getHighestYAtEntry_68bda7eb81b0caf3 = function(arg0) {
        const ret = getObject(arg0).getHighestYAtEntry();
        return ret;
    };
    imports.wbg.__wbg_parentMeasure_85d079e7b2e92567 = function(arg0) {
        const ret = getObject(arg0).parentMeasure;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_graphicalVoiceEntries_dea9c84b7c151ebb = function(arg0, arg1) {
        const ret = getObject(arg1).graphicalVoiceEntries;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_graphicalTies_65ba8014fed8846b = function(arg0, arg1) {
        const ret = getObject(arg1).graphicalTies;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_parentVoiceEntry_db6803c7da34146d = function(arg0) {
        const ret = getObject(arg0).parentVoiceEntry;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_notes_80dc9e9edb061dfd = function(arg0, arg1) {
        const ret = getObject(arg1).notes;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_sourceNote_2af7c5a42396468e = function(arg0) {
        const ret = getObject(arg0).sourceNote;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getSVGGElement_ce2a71516aa61312 = function(arg0) {
        const ret = getObject(arg0).getSVGGElement();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getStemSVG_07f004397ba5cc68 = function(arg0) {
        const ret = getObject(arg0).getStemSVG();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getBeamSVGs_75777c191e0993bb = function(arg0, arg1) {
        const ret = getObject(arg1).getBeamSVGs();
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_startNote_767afaba13b5e1d1 = function(arg0) {
        const ret = getObject(arg0).startNote;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_SVGElement_bdd98510cef13cb1 = function(arg0) {
        const ret = getObject(arg0).SVGElement;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_parentMusicSystem_2d87a9156ac134f7 = function(arg0) {
        const ret = getObject(arg0).parentMusicSystem;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_musicSystems_71f9db9c48b77f7e = function(arg0, arg1) {
        const ret = getObject(arg1).musicSystems;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_SVGElement_b862cfdbc2f2558a = function(arg0) {
        const ret = getObject(arg0).SVGElement;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_slur_8304a6fe37364b92 = function(arg0) {
        const ret = getObject(arg0).slur;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_id_60e49b6624e58fc2 = function(arg0) {
        const ret = getObject(arg0).id;
        return ret;
    };
    imports.wbg.__wbg_staffLines_6eea387afb7bf9e4 = function(arg0, arg1) {
        const ret = getObject(arg1).staffLines;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_graphicalSlurs_166da7eb2a9666ad = function(arg0, arg1) {
        const ret = getObject(arg1).graphicalSlurs;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_absolutePosition_15347b1dcba42f77 = function(arg0) {
        const ret = getObject(arg0).absolutePosition;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_x_fbae95a496535e5c = function(arg0) {
        const ret = getObject(arg0).x;
        return ret;
    };
    imports.wbg.__wbg_setNewSystemAtXMLNewPageAttribute_a850b05afa3e9114 = function(arg0, arg1) {
        getObject(arg0).NewSystemAtXMLNewPageAttribute = arg1 !== 0;
    };
    imports.wbg.__wbg_setNewSystemAtXMLNewSystemAttribute_85f4a9c80cd85080 = function(arg0, arg1) {
        getObject(arg0).NewSystemAtXMLNewSystemAttribute = arg1 !== 0;
    };
    imports.wbg.__wbg_reset_f3e9c253e4b14d0c = function(arg0) {
        getObject(arg0).reset();
    };
    imports.wbg.__wbg_show_4cf577f903a075f2 = function(arg0) {
        getObject(arg0).show();
    };
    imports.wbg.__wbg_hide_9d8a545faf194218 = function(arg0) {
        getObject(arg0).hide();
    };
    imports.wbg.__wbg_next_2f65d938ae2296be = function(arg0) {
        getObject(arg0).next();
    };
    imports.wbg.__wbg_previous_8dcfc92d3457d593 = function(arg0) {
        getObject(arg0).previous();
    };
    imports.wbg.__wbg_iterator_f02878e92d31b011 = function(arg0) {
        const ret = getObject(arg0).iterator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_cursorElement_ba9f3b604c5c6821 = function(arg0) {
        const ret = getObject(arg0).cursorElement;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_endReached_2f179c55a301b6e6 = function(arg0) {
        const ret = getObject(arg0).endReached;
        return ret;
    };
    imports.wbg.__wbg_currentVoiceEntries_44382659d766d76e = function(arg0, arg1) {
        const ret = getObject(arg1).currentVoiceEntries;
        var ptr1 = isLikeNone(ret) ? 0 : passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        var len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_currentTimeStamp_2d89fc8170427859 = function(arg0) {
        const ret = getObject(arg0).currentTimeStamp;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_notes_58998a128263c031 = function(arg0, arg1) {
        const ret = getObject(arg1).notes;
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_parentVoice_3f05de91d794ff94 = function(arg0) {
        const ret = getObject(arg0).parentVoice;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_parentSourceStaffEntry_a71c14b588952dd5 = function(arg0) {
        const ret = getObject(arg0).parentSourceStaffEntry;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_timestamp_0f124b2d902da427 = function(arg0) {
        const ret = getObject(arg0).timestamp;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_voiceId_8c385cd605f459f2 = function(arg0) {
        const ret = getObject(arg0).voiceId;
        return ret;
    };
    imports.wbg.__wbg_isRest_24efd610bb44f238 = function(arg0) {
        const ret = getObject(arg0).isRest();
        return ret;
    };
    imports.wbg.__wbg_length_8b948a834c69bf7e = function(arg0) {
        const ret = getObject(arg0).length;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_pitch_8af9f6739f7077a4 = function(arg0) {
        const ret = getObject(arg0).pitch;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_tie_7cbbd17ba98d67b4 = function(arg0) {
        const ret = getObject(arg0).tie;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_voiceEntry_e5f95611ad4838c2 = function(arg0) {
        const ret = getObject(arg0).voiceEntry;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_sourceMeasure_48c1f6e6c6f12ec9 = function(arg0) {
        const ret = getObject(arg0).sourceMeasure;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_halfTone_662d22fa3ac15cde = function(arg0) {
        const ret = getObject(arg0).halfTone;
        return ret;
    };
    imports.wbg.__wbg_Duration_3f5ece45873ac241 = function(arg0) {
        const ret = getObject(arg0).Duration;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_Pitch_8697ffb226bf4b1e = function(arg0) {
        const ret = getObject(arg0).Pitch;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_StartNote_0d598410a5023046 = function(arg0) {
        const ret = getObject(arg0).StartNote;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_StartNote_f9aae68029c7308b = function(arg0) {
        const ret = getObject(arg0).StartNote;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_numerator_a2eae54c4565d7dc = function(arg0) {
        const ret = getObject(arg0).numerator;
        return ret;
    };
    imports.wbg.__wbg_denominator_620bb6583ddff6b7 = function(arg0) {
        const ret = getObject(arg0).denominator;
        return ret;
    };
    imports.wbg.__wbg_wholeValue_e42e169ed43f2213 = function(arg0) {
        const ret = getObject(arg0).wholeValue;
        return ret;
    };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function(arg0, arg1) {
        var v0 = getCachedStringFromWasm0(arg0, arg1);
    if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
    console.error(v0);
};
imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
    const ret = new Error();
    return addHeapObject(ret);
};
imports.wbg.__wbg_stack_658279fe44541cf6 = function(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbindgen_cb_drop = function(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};
imports.wbg.__wbindgen_number_new = function(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};
imports.wbg.__wbg_set_f975102236d3c502 = function(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
};
imports.wbg.__wbindgen_is_falsy = function(arg0) {
    const ret = !getObject(arg0);
    return ret;
};
imports.wbg.__wbg_instanceof_Window_f401953a2cf86220 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_document_5100775d18896c16 = function(arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_localStorage_e381d34d0c40c761 = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).localStorage;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_sessionStorage_0a3bf8c7b2253d1a = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).sessionStorage;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_requestAnimationFrame_549258cfa66011f0 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    return ret;
}, arguments) };
imports.wbg.__wbg_clearTimeout_ba63ae54a36e111e = function(arg0, arg1) {
    getObject(arg0).clearTimeout(arg1);
};
imports.wbg.__wbg_fetch_c4b6afebdb1f918e = function(arg0, arg1) {
    const ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_setTimeout_c172d5704ef82276 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
    return ret;
}, arguments) };
imports.wbg.__wbg_body_edb1908d3ceff3a1 = function(arg0) {
    const ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_createComment_354ccab4fdc521ee = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createComment(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_createDocumentFragment_8c86903bbb0a3c3c = function(arg0) {
    const ret = getObject(arg0).createDocumentFragment();
    return addHeapObject(ret);
};
imports.wbg.__wbg_createElement_8bae7856a4bb7411 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createElement(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createElementNS_556a62fb298be5a2 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    const ret = getObject(arg0).createElementNS(v0, v1);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createTextNode_0c38fd80a5b2284d = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createTextNode(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_getElementById_c369ff43f0db99cf = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getElementById(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_setdata_8c2b43af041cc1b3 = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).data = v0;
};
imports.wbg.__wbg_gain_992e52c27fe23f6b = function(arg0) {
    const ret = getObject(arg0).gain;
    return addHeapObject(ret);
};
imports.wbg.__wbg_close_a994f9425dab445c = function() { return handleError(function (arg0) {
    getObject(arg0).close();
}, arguments) };
imports.wbg.__wbg_enqueue_ea194723156c0cc2 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).enqueue(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_append_f7fa3534fc158323 = function() { return handleError(function (arg0, arg1, arg2) {
    getObject(arg0).append(getObject(arg1), getObject(arg2));
}, arguments) };
imports.wbg.__wbg_get_58f6d5f6aee3f846 = function(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_addEventListener_53b787075bd5e003 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).addEventListener(v0, getObject(arg3));
}, arguments) };
imports.wbg.__wbg_addEventListener_4283b15b4f039eb5 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).addEventListener(v0, getObject(arg3), getObject(arg4));
}, arguments) };
imports.wbg.__wbg_dispatchEvent_63c0c01600a98fd2 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).dispatchEvent(getObject(arg1));
    return ret;
}, arguments) };
imports.wbg.__wbg_removeEventListener_92cb9b3943463338 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).removeEventListener(v0, getObject(arg3));
}, arguments) };
imports.wbg.__wbg_removeEventListener_f3689e55cc5b09c4 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).removeEventListener(v0, getObject(arg3), getObject(arg4));
}, arguments) };
imports.wbg.__wbg_getwithindex_6f132a79d56e81ec = function(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_instanceof_Response_849eb93e75734b6e = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_arrayBuffer_29931d52c7206b02 = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).arrayBuffer();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_key_dc5b86010fc4d525 = function(arg0, arg1) {
    const ret = getObject(arg1).key;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_new_4c501d7c115d20a6 = function() { return handleError(function () {
    const ret = new URLSearchParams();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_AudioBuffer_94e5a804c89431db = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof AudioBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_parentNode_6be3abff20e1a5fb = function(arg0) {
    const ret = getObject(arg0).parentNode;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_childNodes_118168e8b23bcb9b = function(arg0) {
    const ret = getObject(arg0).childNodes;
    return addHeapObject(ret);
};
imports.wbg.__wbg_previousSibling_9708a091a3e6e03b = function(arg0) {
    const ret = getObject(arg0).previousSibling;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_nextSibling_709614fdb0fb7a66 = function(arg0) {
    const ret = getObject(arg0).nextSibling;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_textContent_8e392d624539e731 = function(arg0, arg1) {
    const ret = getObject(arg1).textContent;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_settextContent_d271bab459cbb1ba = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).textContent = v0;
};
imports.wbg.__wbg_appendChild_580ccb11a660db68 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_cloneNode_e19c313ea20d5d1d = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).cloneNode();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_getItem_164e8e5265095b87 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    const ret = getObject(arg1).getItem(v0);
    var ptr2 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len2;
    getInt32Memory0()[arg0 / 4 + 0] = ptr2;
}, arguments) };
imports.wbg.__wbg_setItem_ba2bb41d73dac079 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    getObject(arg0).setItem(v0, v1);
}, arguments) };
imports.wbg.__wbg_setbuffer_1793c076d39c6617 = function(arg0, arg1) {
    getObject(arg0).buffer = getObject(arg1);
};
imports.wbg.__wbg_playbackRate_464e22b0beb542df = function(arg0) {
    const ret = getObject(arg0).playbackRate;
    return addHeapObject(ret);
};
imports.wbg.__wbg_start_a0650e3bc8fe8e8d = function() { return handleError(function (arg0) {
    getObject(arg0).start();
}, arguments) };
imports.wbg.__wbg_stop_8cd79bd9477aae21 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).stop(arg1);
}, arguments) };
imports.wbg.__wbg_add_dcb05a8ba423bdac = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).add(v0);
}, arguments) };
imports.wbg.__wbg_remove_698118fb25ab8150 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).remove(v0);
}, arguments) };
imports.wbg.__wbg_altKey_2e6c34c37088d8b1 = function(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};
imports.wbg.__wbg_ctrlKey_bb5b6fef87339703 = function(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};
imports.wbg.__wbg_shiftKey_5911baf439ab232b = function(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};
imports.wbg.__wbg_metaKey_6bf4ae4e83a11278 = function(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};
imports.wbg.__wbg_repeat_f64b916c6eed0685 = function(arg0) {
    const ret = getObject(arg0).repeat;
    return ret;
};
imports.wbg.__wbg_key_dccf9e8aa1315a8e = function(arg0, arg1) {
    const ret = getObject(arg1).key;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_view_7f0ce470793a340f = function(arg0) {
    const ret = getObject(arg0).view;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_respond_b1a43b2e3a06d525 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).respond(arg1 >>> 0);
}, arguments) };
imports.wbg.__wbg_new_ab6fd82b10560829 = function() { return handleError(function () {
    const ret = new Headers();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_open_9c89a087d1313c4c = function(arg0) {
    const ret = getObject(arg0).open;
    return ret;
};
imports.wbg.__wbg_checked_749a34774f2df2e3 = function(arg0) {
    const ret = getObject(arg0).checked;
    return ret;
};
imports.wbg.__wbg_files_8b6e6eff43af0f6d = function(arg0) {
    const ret = getObject(arg0).files;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_value_47fe6384562f52ab = function(arg0, arg1) {
    const ret = getObject(arg1).value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_valueAsNumber_b6456a4b05234115 = function(arg0) {
    const ret = getObject(arg0).valueAsNumber;
    return ret;
};
imports.wbg.__wbg_url_7807f6a1fddc3e23 = function(arg0, arg1) {
    const ret = getObject(arg1).url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_newwithstr_36b0b3f97efe096f = function() { return handleError(function (arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Request(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_newwithstrandinit_3fd6fba4083ff2d0 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Request(v0, getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_Element_6945fc210db80ea9 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Element;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_namespaceURI_5235ee79fd5f6781 = function(arg0, arg1) {
    const ret = getObject(arg1).namespaceURI;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_tagName_a98846fa0b63dd7f = function(arg0, arg1) {
    const ret = getObject(arg1).tagName;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_classList_1f0528ee002e56d4 = function(arg0) {
    const ret = getObject(arg0).classList;
    return addHeapObject(ret);
};
imports.wbg.__wbg_setinnerHTML_26d69b59e1af99c7 = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).innerHTML = v0;
};
imports.wbg.__wbg_outerHTML_e073aa84e7bc1eaf = function(arg0, arg1) {
    const ret = getObject(arg1).outerHTML;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_getAttribute_99bddb29274b29b9 = function(arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    const ret = getObject(arg1).getAttribute(v0);
    var ptr2 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len2;
    getInt32Memory0()[arg0 / 4 + 0] = ptr2;
};
imports.wbg.__wbg_getElementsByTagName_ea632875268a272f = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getElementsByTagName(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_removeAttribute_1b10a06ae98ebbd1 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).removeAttribute(v0);
}, arguments) };
imports.wbg.__wbg_scrollIntoView_eec424449e6c23da = function(arg0, arg1) {
    getObject(arg0).scrollIntoView(getObject(arg1));
};
imports.wbg.__wbg_setAttribute_3c9f6c303b696daa = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    getObject(arg0).setAttribute(v0, v1);
}, arguments) };
imports.wbg.__wbg_before_210596e44d88649f = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).before(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_remove_49b0a5925a04b955 = function(arg0) {
    getObject(arg0).remove();
};
imports.wbg.__wbg_target_2fc177e386c8b7b0 = function(arg0) {
    const ret = getObject(arg0).target;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_cancelBubble_c0aa3172524eb03c = function(arg0) {
    const ret = getObject(arg0).cancelBubble;
    return ret;
};
imports.wbg.__wbg_composedPath_58473fd5ae55f2cd = function(arg0) {
    const ret = getObject(arg0).composedPath();
    return addHeapObject(ret);
};
imports.wbg.__wbg_preventDefault_b1a4aafc79409429 = function(arg0) {
    getObject(arg0).preventDefault();
};
imports.wbg.__wbg_debug_5fb96680aecf5dc8 = function(arg0) {
    console.debug(getObject(arg0));
};
imports.wbg.__wbg_error_8e3928cfb8a43e2b = function(arg0) {
    console.error(getObject(arg0));
};
imports.wbg.__wbg_info_530a29cb2e4e3304 = function(arg0) {
    console.info(getObject(arg0));
};
imports.wbg.__wbg_log_5bb5f88f245d7762 = function(arg0) {
    console.log(getObject(arg0));
};
imports.wbg.__wbg_warn_63bbae1730aead09 = function(arg0) {
    console.warn(getObject(arg0));
};
imports.wbg.__wbg_arrayBuffer_307ddd1bd1d04e23 = function(arg0) {
    const ret = getObject(arg0).arrayBuffer();
    return addHeapObject(ret);
};
imports.wbg.__wbg_length_d0a802565d17eec4 = function(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};
imports.wbg.__wbg_byobRequest_72fca99f9c32c193 = function(arg0) {
    const ret = getObject(arg0).byobRequest;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_close_184931724d961ccc = function() { return handleError(function (arg0) {
    getObject(arg0).close();
}, arguments) };
imports.wbg.__wbg_instanceof_ShadowRoot_9db040264422e84a = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ShadowRoot;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_host_c667c7623404d6bf = function(arg0) {
    const ret = getObject(arg0).host;
    return addHeapObject(ret);
};
imports.wbg.__wbg_destination_0014df38da590ed6 = function(arg0) {
    const ret = getObject(arg0).destination;
    return addHeapObject(ret);
};
imports.wbg.__wbg_currentTime_9bc85e1579050a3f = function(arg0) {
    const ret = getObject(arg0).currentTime;
    return ret;
};
imports.wbg.__wbg_new_2f044fe84595e924 = function() { return handleError(function () {
    const ret = new lAudioContext();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createBufferSource_0d65cd58ccd38511 = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).createBufferSource();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_createGain_074e52e83a75da7f = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).createGain();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_decodeAudioData_6e4a09d9bdf064dc = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).decodeAudioData(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_setvalue_386ea64eb220ce1c = function(arg0, arg1) {
    getObject(arg0).value = arg1;
};
imports.wbg.__wbg_linearRampToValueAtTime_e88770a6b173b85e = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).linearRampToValueAtTime(arg1, arg2);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_setValueAtTime_1c6d27faeb36996a = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).setValueAtTime(arg1, arg2);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_detail_40fcf009a8ab1ab9 = function(arg0) {
    const ret = getObject(arg0).detail;
    return addHeapObject(ret);
};
imports.wbg.__wbg_newwitheventinitdict_df944b5b582e7ecf = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new CustomEvent(v0, getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_search_c68f506c44be6d1e = function(arg0, arg1) {
    const ret = getObject(arg1).search;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_setsearch_fd62f4de409a2bb3 = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).search = v0;
};
imports.wbg.__wbg_new_67853c351755d2cf = function() { return handleError(function (arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new URL(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_WorkerGlobalScope_46b577f151fad960 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof WorkerGlobalScope;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_fetch_921fad6ef9e883dd = function(arg0, arg1) {
    const ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_connect_186433827476e7d8 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).connect(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbindgen_is_function = function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
};
imports.wbg.__wbg_queueMicrotask_481971b0d87f3dd4 = function(arg0) {
    queueMicrotask(getObject(arg0));
};
imports.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6 = function(arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
};
imports.wbg.__wbg_get_bd8e338fbd5f5cc8 = function(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};
imports.wbg.__wbg_instanceof_Error_e20bb56fd5591a93 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Error;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_new_28c511d9baebfa89 = function(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Error(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_message_5bf28016c2b49cfb = function(arg0) {
    const ret = getObject(arg0).message;
    return addHeapObject(ret);
};
imports.wbg.__wbg_name_e7429f0dda6079e2 = function(arg0) {
    const ret = getObject(arg0).name;
    return addHeapObject(ret);
};
imports.wbg.__wbg_toString_ffe4c9ea3b3532e9 = function(arg0) {
    const ret = getObject(arg0).toString();
    return addHeapObject(ret);
};
imports.wbg.__wbg_newnoargs_e258087cd0daa0ea = function(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Function(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_call_27c0f87801dedf93 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_call_b3ca7c6051f9bec1 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_now_3014639a94423537 = function() {
    const ret = Date.now();
    return ret;
};
imports.wbg.__wbg_is_010fdc0f4ab96916 = function(arg0, arg1) {
    const ret = Object.is(getObject(arg0), getObject(arg1));
    return ret;
};
imports.wbg.__wbg_new_72fb9a18b5ae2624 = function() {
    const ret = new Object();
    return addHeapObject(ret);
};
imports.wbg.__wbg_toString_c816a20ab859d0c1 = function(arg0) {
    const ret = getObject(arg0).toString();
    return addHeapObject(ret);
};
imports.wbg.__wbg_fromCharCode_94619b7d46827d3d = function(arg0, arg1) {
    const ret = String.fromCharCode(...getArrayU16FromWasm0(arg0, arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_new_81740750da40724f = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_485(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = state0.b = 0;
    }
};
imports.wbg.__wbg_resolve_b0083a7967828ec8 = function(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};
imports.wbg.__wbg_then_0c86a60e8fcfe9f6 = function(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_then_a73caa9a87991566 = function(arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};
imports.wbg.__wbg_globalThis_d1e6af4856ba331b = function() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_self_ce0dbfc45cf2f5be = function() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_window_c6fb939a7f436783 = function() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_global_207b558942527489 = function() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_new_63b92bc8671ed464 = function(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};
imports.wbg.__wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb = function(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_buffer_dd7f74bc60f1faab = function(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};
imports.wbg.__wbg_length_c20a40f15020d68a = function(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};
imports.wbg.__wbg_byteLength_58f7b4fab1919d44 = function(arg0) {
    const ret = getObject(arg0).byteLength;
    return ret;
};
imports.wbg.__wbg_byteOffset_81d60f7392524f62 = function(arg0) {
    const ret = getObject(arg0).byteOffset;
    return ret;
};
imports.wbg.__wbg_set_a47bac70306a19a7 = function(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};
imports.wbg.__wbg_buffer_12d079cc21e14bdb = function(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};
imports.wbg.__wbg_get_e3c254076557e348 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_has_0af94d20077affa2 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(getObject(arg0), getObject(arg1));
    return ret;
}, arguments) };
imports.wbg.__wbg_set_1f9b04f170055d33 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments) };
imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};
imports.wbg.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};
imports.wbg.__wbindgen_rethrow = function(arg0) {
    throw takeObject(arg0);
};
imports.wbg.__wbindgen_memory = function() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper1074 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 390, __wbg_adapter_32);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper1076 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 386, __wbg_adapter_35);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper1078 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 388, __wbg_adapter_38);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper6594 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 544, __wbg_adapter_41);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper6596 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 538, __wbg_adapter_44);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper6598 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 546, __wbg_adapter_47);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper6600 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 542, __wbg_adapter_50);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper6602 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 540, __wbg_adapter_53);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper6604 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 536, __wbg_adapter_56);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper16486 = function(arg0, arg1, arg2) {
    const ret = makeClosure(arg0, arg1, 1530, __wbg_adapter_59);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper17540 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1602, __wbg_adapter_62);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper18248 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1656, __wbg_adapter_65);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper23979 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1700, __wbg_adapter_68);
    return addHeapObject(ret);
};

return imports;
}

function __wbg_init_memory(imports, maybe_memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedInt32Memory0 = null;
    cachedUint16Memory0 = null;
    cachedUint32Memory0 = null;
    cachedUint8Memory0 = null;

    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(input) {
    if (wasm !== undefined) return wasm;

    if (typeof input === 'undefined') {
        input = new URL('magic_piano_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await input, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync }
export default __wbg_init;
