(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
__webpack_require__.exportStar(__webpack_require__("./bar.js"), exports);
const a = 'foo';
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _foo = __webpack_require__("./foo.js");
console.log(_foo.a);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);