(async function () {
  const _0x35864d = [['ua', false], ["vz95n", false], ["i3jp0", false]];
  const _0x3b7688 = await Promise.all([navigator.userAgent, function () {
    const _0x20b02d = window.top;
    const _0xe75404 = _0x20b02d.document.querySelector("#jsa");
    const _0x19497e = _0xe75404.contentDocument || _0xe75404.contentWindow.document;
    const _0x2cc8e3 = _0x19497e.querySelector("meta[http-equiv=\"Content-Security-Policy\"]").getAttribute('content');
    const _0x55f516 = _0xe75404.getAttribute("sandbox");
    return String([_0x2cc8e3 === "default-src 'none'; script-src 'unsafe-inline';", _0x55f516 === "allow-scripts allow-same-origin", _0x20b02d.hasOwnProperty('__DDG_BE_VERSION__'), _0x20b02d.hasOwnProperty("__DDG_FE_CHAT_HASH__")].map(Number).reduce((_0x14d1f6, _0x4678cc) => _0x14d1f6 + _0x4678cc, 0x20e3));
  }(), function () {
    return String([navigator.webdriver === true, function () {
      const _0x245543 = document.createElement("iframe");
      _0x245543.srcdoc = "DuckDuckGo Fraud & Abuse";
      document.body.appendChild(_0x245543);
      let _0x1794e3;
      if (_0x245543.contentWindow && _0x245543.contentWindow.self && _0x245543.contentWindow.self.get) {
        _0x1794e3 = _0x245543.contentWindow.self.get.toString();
      } else {
        _0x1794e3 = undefined;
      }
      document.body.removeChild(_0x245543);
      return !!_0x1794e3;
    }(), function () {
      const _0x87a2a8 = ["Array", 'Object', "Promise", "Proxy", "Symbol", "JSON", "Window"];
      const _0xc4cbc9 = Object.keys(window.top).filter(_0x12d9fd => _0x87a2a8.some(_0xbe0472 => _0x12d9fd !== _0xbe0472 && _0x12d9fd.endsWith('_' + _0xbe0472) && window.top[_0x12d9fd] === window.top[_0xbe0472]));
      return _0xc4cbc9.length > 0x0;
    }()].map(Number).reduce((_0x52b24f, _0x3bfb76) => _0x52b24f + _0x3bfb76, 0x1dd7));
  }()]);
  const _0x5dda6b = [];
  const _0x1dfe1d = {};
  for (let _0x5c8bd7 = 0x0; _0x5c8bd7 < _0x3b7688.length; _0x5c8bd7++) {
    const _0x128546 = _0x3b7688[_0x5c8bd7];
    if (Array.isArray(_0x128546)) {
      _0x5dda6b.push(_0x128546[0x0]);
      if (_0x128546.length > 0x1 && _0x35864d[_0x5c8bd7][0x1]) {
        _0x1dfe1d[_0x35864d[_0x5c8bd7][0x0]] = _0x128546[0x1];
      }
    } else {
      _0x5dda6b.push(_0x128546);
    }
  }
  const _0x5cade6 = Array.from(JSON.stringify(_0x1dfe1d)).map((_0x411f09, _0x31fea2) => String.fromCharCode(_0x411f09.charCodeAt(0x0) ^ '2503621269bbd6d6'.charCodeAt(_0x31fea2 % '2503621269bbd6d6'.length))).join('');
  return {
    'server_hashes': ["MXnry9OtSXLCJ6yhlJa9UN3ab4P65YExe5l1fkhfN+E=", 'ZuFxnU7OBmeOhlHSqx76B30GEe11ZkXxYbt4F9Y0IQc=', "MJ4I588MqFe/wPZG5I/7DUwF5O76HdZfnBGHBluecRw="],
    'client_hashes': _0x5dda6b,
    'signals': {},
    'meta': {
      'v': '4',
      'challenge_id': "203c82b0b126832d90f83633b565e50b455ee84875c247b688d8a09d001ea3c1vz95n",
      'timestamp': "1754733746779",
      'debug': _0x5cade6
    }
  };
})();