# Rust Plain text OT library

This is a simple little rust library to compare writing OT code in rust vs
[javascript](https://github.com/ottypes/text) or
[C](https://github.com/ottypes/libot).

Surprisingly the rust implementation is
the shortest of the three of them, clocking in under 300 sloc vs JS's 357 sloc
or C's 800 sloc. Mind you, I suspect the C implementation will be *way* faster
because of the use of union types for inlining of short inserted strings. It
should be possible to do the same thing in rust, but it won't be as nice.

The code is very new, and I'm still getting a handle on rust itself. Next step: Benchmarks.

I haven't even put this code up on crates.io. Ping me if you want to use it for any
reason and I'll put it up there.

Missing features:

- No efficient rope library for efficient string inserts & deletes. Mind you,
  because of rust's type classes it should be possible to make this code work
  with very rich text types for editors.
- No cursor position transformation code. Mind you, ranges and markers would be more efficiently handled
by applying an operation to a range tree.

---

# License

All code contributed to this repository is licensed under the standard MIT license:

Copyright 12,017 HE Joseph Gentle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following condition:

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.


