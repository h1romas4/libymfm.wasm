// TextEncoder/TextDecoder polyfills for utf-8 - an implementation of TextEncoder/TextDecoder APIs
// Written in 2013 by Viktor Mukhachev <vic99999@yandex.ru>
// To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
// You should have received a copy of the CC0 Public Domain Dedication along with this software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// Some important notes about the polyfill below:
// Native TextEncoder/TextDecoder implementation is overwritten
// String.prototype.codePointAt polyfill not included, as well as String.fromCodePoint
// TextEncoder.prototype.encode returns a regular array instead of Uint8Array
// No options (fatal of the TextDecoder constructor and stream of the TextDecoder.prototype.decode method) are supported.
// TextDecoder.prototype.decode does not valid byte sequences
// This is a demonstrative implementation not intended to have the best performance

// http://encoding.spec.whatwg.org/#textencoder

// http://encoding.spec.whatwg.org/#textencoder

export function TextEncoder() {
}

TextEncoder.prototype.encode = function (string) {
  var octets = [];
  var length = string.length;
  var i = 0;
  while (i < length) {
    var codePoint = string.codePointAt(i);
    var c = 0;
    var bits = 0;
    if (codePoint <= 0x0000007F) {
      c = 0;
      bits = 0x00;
    } else if (codePoint <= 0x000007FF) {
      c = 6;
      bits = 0xC0;
    } else if (codePoint <= 0x0000FFFF) {
      c = 12;
      bits = 0xE0;
    } else if (codePoint <= 0x001FFFFF) {
      c = 18;
      bits = 0xF0;
    }
    octets.push(bits | (codePoint >> c));
    c -= 6;
    while (c >= 0) {
      octets.push(0x80 | ((codePoint >> c) & 0x3F));
      c -= 6;
    }
    i += codePoint >= 0x10000 ? 2 : 1;
  }
  return octets;
};

export function TextDecoder() {
}

TextDecoder.prototype.decode = function (octets) {
  if(octets == null) return; // wasm-bindgen hack
  var string = "";
  var i = 0;
  while (i < octets.length) {
    var octet = octets[i];
    var bytesNeeded = 0;
    var codePoint = 0;
    if (octet <= 0x7F) {
      bytesNeeded = 0;
      codePoint = octet & 0xFF;
    } else if (octet <= 0xDF) {
      bytesNeeded = 1;
      codePoint = octet & 0x1F;
    } else if (octet <= 0xEF) {
      bytesNeeded = 2;
      codePoint = octet & 0x0F;
    } else if (octet <= 0xF4) {
      bytesNeeded = 3;
      codePoint = octet & 0x07;
    }
    if (octets.length - i - bytesNeeded > 0) {
      var k = 0;
      while (k < bytesNeeded) {
        octet = octets[i + k + 1];
        codePoint = (codePoint << 6) | (octet & 0x3F);
        k += 1;
      }
    } else {
      codePoint = 0xFFFD;
      bytesNeeded = octets.length - i;
    }
    string += String.fromCodePoint(codePoint);
    i += bytesNeeded + 1;
  }
  return string
};
