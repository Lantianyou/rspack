(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _stuff = __webpack_require__("./stuff.js");
(0, _stuff.bar)();
var f = (0, _stuff.baz)();
f();
function getClass() {
    class MyClass {
    }
    return MyClass;
}
console.log(getClass().name);
},
"./stuff.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    bar: ()=>bar,
    baz: ()=>Baz
});
function bar() {
    console.log("outer bar");
}
function Baz() {
    function bar() {
        console.log("inner bar");
    }
    function bog() {
        console.log("inner bog");
    }
    return bar(), bog;
}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);