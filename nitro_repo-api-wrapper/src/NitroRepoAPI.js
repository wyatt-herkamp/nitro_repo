"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAPIError = exports.INTERNAL_ERROR = exports.NOT_AUTHORIZED = exports.INVALID_LOGIN = exports.init = exports.apiClient = exports.apiURL = void 0;
var axios_1 = __importDefault(require("axios"));
exports.apiClient = axios_1.default.create({
    headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
    },
});
function init(myURL) {
    exports.apiURL = myURL;
    exports.apiClient = axios_1.default.create({
        baseURL: myURL,
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
    });
}
exports.init = init;
exports.INVALID_LOGIN = {
    user_friendly_message: "Invalid Username or Password",
    code: 401,
};
exports.NOT_AUTHORIZED = {
    user_friendly_message: "Not Authorized for that action",
    code: 401,
};
exports.INTERNAL_ERROR = {
    user_friendly_message: "Internal Error Occured ",
    code: 500,
};
function createAPIError(code, message) {
    var value = {
        user_friendly_message: message,
        code: code,
    };
    return value;
}
exports.createAPIError = createAPIError;
//# sourceMappingURL=NitroRepoAPI.js.map