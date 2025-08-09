(async function () {
  const _0x4e90ee = [['ua', false], ['h8jbt', false], ["i3jp0", false]];
  const _0x214df8 = await Promise.all([navigator.userAgent, function () {
    const _0x2fae0e = document.createElement("div");
    _0x2fae0e.innerHTML = '<div><div></div><div></div';
    return String(0x435 + _0x2fae0e.innerHTML.length * _0x2fae0e.querySelectorAll('*').length);
  }(), function () {
    return String([navigator.webdriver === true, function () {
      const _0x130ab4 = document.createElement("iframe");
      _0x130ab4.srcdoc = "DuckDuckGo\\x20Fraud\\x20&\\x20Abuse";
      document.body.appendChild(_0x130ab4);
      let _0x4338d3;
      if (_0x130ab4.contentWindow && _0x130ab4.contentWindow.self && _0x130ab4.contentWindow.self.get) {
        _0x4338d3 = _0x130ab4.contentWindow.self.get.toString();
      } else {
        _0x4338d3 = undefined;
      }
      document.body.removeChild(_0x130ab4);
      return !!_0x4338d3;
    }(), function () {
      const _0x314243 = ["Array", "Object", "Promise", 'Proxy', "Symbol", "JSON", "Window"];
      const _0x14dac3 = Object.keys(window.top).filter(_0x54efa5 => _0x314243.some(_0x312d19 => _0x54efa5 !== _0x312d19 && _0x54efa5.endsWith('_' + _0x312d19) && window.top[_0x54efa5] === window.top[_0x312d19]));
      return _0x14dac3.length > 0x0;
    }()].map(Number).reduce((_0xccad8f, _0x2933b9) => _0xccad8f + _0x2933b9, 0x10f6));
  }()]);
  const _0x301931 = [];
  const _0x14c3f6 = {};
  for (let _0x428189 = 0x0; _0x428189 < _0x214df8.length; _0x428189++) {
    const _0x27312a = _0x214df8[_0x428189];
    if (Array.isArray(_0x27312a)) {
      _0x301931.push(_0x27312a[0x0]);
      if (_0x27312a.length > 0x1 && _0x4e90ee[_0x428189][0x1]) {
        _0x14c3f6[_0x4e90ee[_0x428189][0x0]] = _0x27312a[0x1];
      }
    } else {
      _0x301931.push(_0x27312a);
    }
  }
  const _0x448b1d = Array.from(JSON.stringify(_0x14c3f6)).map((_0x26f08f, _0x19c64d) => String.fromCharCode(_0x26f08f.charCodeAt(0x0) ^ '42dcf46eae028a66'.charCodeAt(_0x19c64d % '42dcf46eae028a66'.length))).join('');
  return {
    'server_hashes': ["5gjsUrKZNDEQ8P0iLiNQCp/z8jKc6Qy28/BTshgIGrk=", 'U28zQNudW5nfzjsT02xg7QledUe8pSFXBGZpdqdAa/g=', "jhTr8NensW3XImOJOWWx9rWv1DjDuTKkQSJLtzjY7jQ="],
    'client_hashes': _0x301931,
    'signals': {},
    'meta': {
      'v': '4',
      'challenge_id': "3f592fdc9da387d8bec3b81a99629e2b7ead8395e32c920d8ace475a66280b3ch8jbt",
      'timestamp': "1754714071441",
      'debug': _0x448b1d
    }
  };
})();