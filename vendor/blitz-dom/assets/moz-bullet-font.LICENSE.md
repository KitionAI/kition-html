# moz-bullet-font.ttf

`moz-bullet-font.ttf` is "Mozilla Bullet" version 1.1, the `-moz-bullet-font` of Mozilla's Gecko
engine, designed by Mats Palmgren. `BULLET_FONT` in `../src/lib.rs` embeds it, and `default.css`
embeds the same font as WOFF2 in its `-moz-bullet-font` `@font-face`, as Gecko's `html.css` does.

Kition HTML does not draw with it: the engine paints the disc, circle, square and disclosure
markers as shapes (`blitz-paint/src/text_markers.rs`) and gives Blitz its own font context, so the
default context that registers `BULLET_FONT` is never built. The font ships because the vendored
Blitz sources embed it.

## Source

Both files are byte copies from mozilla-central changeset
[`7d5056c95c12`](https://hg.mozilla.org/mozilla-central/rev/7d5056c95c1264f9e0379a340f6c9dc97c5a9237)
(Bug 1903922, "Add an upward-pointing triangle glyph to -moz-bullet-font", Jonathan Kew,
2024-06-24), the last change to these files in mozilla-central (checked on 2026-11-16):

| File here | mozilla-central file at `7d5056c95c12` | Bytes | SHA-256 |
|---|---|---|---|
| `moz-bullet-font.ttf` | [`layout/style/res/Mozilla_Bullet.ttf`](https://hg.mozilla.org/mozilla-central/raw-file/7d5056c95c12/layout/style/res/Mozilla_Bullet.ttf) | 2536 | `eb82b69861f3424eeef2e6c3863c19ee4da5bfc68293760c4d16348b694af75e` |
| WOFF2 in `default.css` | [`layout/style/res/Mozilla_Bullet.woff2`](https://hg.mozilla.org/mozilla-central/raw-file/7d5056c95c12/layout/style/res/Mozilla_Bullet.woff2), as base64 in `layout/style/res/html.css` | 1180 | `64509a32feeb777a3b2850ea06757782c101ab36ead80183a7509bbc59af6545` |

The font's source form is the BirdFont file
[`layout/style/res/Mozilla_Bullet.bf`](https://hg.mozilla.org/mozilla-central/file/7d5056c95c12/layout/style/res/Mozilla_Bullet.bf)
at the same changeset. The outlines are TrueType, as Mozilla ships them; nothing was converted.

Version 1.1 replaced version 1.0 here (the CFF `moz-bullet-font.otf` that Blitz shipped, and
Gecko's 1.0 WOFF2 in `default.css`). It adds U+25B4 and redraws U+25E6 within one font unit; the
other glyphs' outlines and every advance width are those of 1.0.

## License

The font names the SIL Open Font License in its metadata: name table entries 13 (license) "SIL
Open Font License" and 14 (license URL) "https://openfontlicense.org", the same as the `license`
and `license_url` of its BirdFont source. It has no copyright line and declares no Reserved Font
Name. That URL publishes version 1.1 of the license, reproduced below.

```text
Mozilla Bullet 1.1, by Mozilla (designer Mats Palmgren).
The font states no copyright line and no Reserved Font Name.

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at:
https://openfontlicense.org


-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
```
