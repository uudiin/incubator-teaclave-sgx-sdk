// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#[allow(dead_code)]
pub mod fallback {
    use core::cmp;
    use core::mem;

    const LO_U64: u64 = 0x0101010101010101;
    const HI_U64: u64 = 0x8080808080808080;

    // use truncation
    const LO_USIZE: usize = LO_U64 as usize;
    const HI_USIZE: usize = HI_U64 as usize;

    /// Return `true` if `x` contains any zero byte.
    ///
    /// From *Matters Computational*, J. Arndt
    ///
    /// "The idea is to subtract one from each of the bytes and then look for
    /// bytes where the borrow propagated all the way to the most significant
    /// bit."
    #[inline]
    fn contains_zero_byte(x: usize) -> bool {
        x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn repeat_byte(b: u8) -> usize {
        let mut rep = (b as usize) << 8 | b as usize;
        rep = rep << 16 | rep;
        rep
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn repeat_byte(b: u8) -> usize {
        let mut rep = (b as usize) << 8 | b as usize;
        rep = rep << 16 | rep;
        rep = rep << 32 | rep;
        rep
    }

    /// Return the first index matching the byte `a` in `text`.
    pub fn memchr(x: u8, text: &[u8]) -> Option<usize> {
        // Scan for a single byte value by reading two `usize` words at a time.
        //
        // Split `text` in three parts
        // - unaligned initial part, before the first word aligned address in text
        // - body, scan by 2 words at a time
        // - the last remaining part, < 2 word size
        let len = text.len();
        let ptr = text.as_ptr();
        let usize_bytes = mem::size_of::<usize>();

        // search up to an aligned boundary
        let mut offset = ptr.align_offset(usize_bytes);
        if offset > 0 {
            offset = cmp::min(offset, len);
            if let Some(index) = text[..offset].iter().position(|elt| *elt == x) {
                return Some(index);
            }
        }

        // search the body of the text
        let repeated_x = repeat_byte(x);

        if len >= 2 * usize_bytes {
            while offset <= len - 2 * usize_bytes {
                unsafe {
                    let u = *(ptr.offset(offset as isize) as *const usize);
                    let v = *(ptr.offset((offset + usize_bytes) as isize) as *const usize);

                    // break if there is a matching byte
                    let zu = contains_zero_byte(u ^ repeated_x);
                    let zv = contains_zero_byte(v ^ repeated_x);
                    if zu || zv {
                        break;
                    }
                }
                offset += usize_bytes * 2;
            }
        }

        // find the byte after the point the body loop stopped
        text[offset..].iter().position(|elt| *elt == x).map(|i| offset + i)
    }

    /// Return the last index matching the byte `a` in `text`.
    pub fn memrchr(x: u8, text: &[u8]) -> Option<usize> {
        // Scan for a single byte value by reading two `usize` words at a time.
        //
        // Split `text` in three parts
        // - unaligned tail, after the last word aligned address in text
        // - body, scan by 2 words at a time
        // - the first remaining bytes, < 2 word size
        let len = text.len();
        let ptr = text.as_ptr();
        let usize_bytes = mem::size_of::<usize>();

        // search to an aligned boundary
        let end_align = (ptr as usize + len) & (usize_bytes - 1);
        let mut offset;
        if end_align > 0 {
            offset = if end_align >= len { 0 } else { len - end_align };
            if let Some(index) = text[offset..].iter().rposition(|elt| *elt == x) {
                return Some(offset + index);
            }
        } else {
            offset = len;
        }

        // search the body of the text
        let repeated_x = repeat_byte(x);

        while offset >= 2 * usize_bytes {
            unsafe {
                let u = *(ptr.offset(offset as isize - 2 * usize_bytes as isize) as *const usize);
                let v = *(ptr.offset(offset as isize - usize_bytes as isize) as *const usize);

                // break if there is a matching byte
                let zu = contains_zero_byte(u ^ repeated_x);
                let zv = contains_zero_byte(v ^ repeated_x);
                if zu || zv {
                    break;
                }
            }
            offset -= 2 * usize_bytes;
        }

        // find the byte before the point the body loop stopped
        text[..offset].iter().rposition(|elt| *elt == x)
    }
}
