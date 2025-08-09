let window = {};
window.top = window;

let navigator = {
    userAgent: nativeGetUserAgent(),
    webdriver: false,
};

let document = {
    body: {
        appendChild: function() {},
        removeChild: function() {},
    },
    createElement: function(name) {
        return nativeCreateElement(name);
    }
};