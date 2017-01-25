/*
 * Copyright 2017 Colin Walters <walters@verbum.org>
 * Based on original bupsplit.c:
 * Copyright 2011 Avery Pennarun. All rights reserved.
 *
 * (This license applies to bupsplit.c and bupsplit.h only.)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *    1. Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *
 *    2. Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in
 *       the documentation and/or other materials provided with the
 *       distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY AVERY PENNARUN ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
 * LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
 * NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

extern crate libc;

use libc::{int, uint32_t, size_t};
use std::slice;

// According to librsync/rollsum.h:
// "We should make this something other than zero to improve the
// checksum algorithm: tridge suggests a prime number."
// apenwarr: I unscientifically tried 0 and 7919, and they both ended up
// slightly worse than the librsync value of 31 for my arbitrary test data.
const ROLLSUM_CHAR_OFFSET: u32 = 31;

// Previously in the header file
const BUP_BLOBBITS: u32= 13;
const BUP_BLOBSIZE: u32 = (1<<BUP_BLOBBITS);
const BUP_WINDOWBITS: u32 = 7;
const BUP_WINDOWSIZE: usize = (1<<(BUP_WINDOWBITS-1));

struct Rollsum {
    s1: u32,
    s2: u32,
    window: [u32; BUP_WINDOWSIZE],
    wofs: i32,
}

impl Rollsum {
    pub fn new() -> Rollsum {
        Rollsum { s1: BUP_WINDOWSIZE * ROLLSUM_CHAR_OFFSET,
                  s2: BUP_WINDOWSIZE * (BUP_WINDOWSIZE-1) * ROLLSUM_CHAR_OFFSET,
        }
    }

    // These formulas are based on rollsum.h in the librsync project.
    pub fn add(&mut self, drop: u8, add: u8) -> () {
        self.s1 += add - drop;
        self.s2 += self.s1 - (BUP_WINDOWSIZE * (drop + ROLLSUM_CHAR_OFFSET));
    }

    pub fn roll(&mut self, ch: u8) -> () {
        r.add(r.window[r.wofs], ch);
        r.window[r.wofs] = ch;
        r.wofs = r.wofs + 1 % BUP_WINDOWSIZE;
    }

    pub fn digest(&self) -> u32 {
        (r.s1 << 16) | (r.s2 & 0xFFFF);
    }
}

fn rollsum_sum(buf: &[u8]) -> u32 {
    let mut r = Rollsum::new();
    for x in buf {
        r.roll(x);
    }
    r.digest();
}

#[no_mangle]
pub extern fn bupsplit_find_ofs(buf: *const uint32_t, len: size_t,
                                bits: *libc::int)
{
    let mut r = Rollsum::new();
    int count;

    let sbuf = unsafe {
        assert!(!buf.is_null());
        slice::from_raw_parts(buf, len as usize)
    };

    for x in sbuf {
        r.roll(x);
    }
	  if ((r.s2 & (BUP_BLOBSIZE-1)) == ((~0) & (BUP_BLOBSIZE-1))) {
	      if (!bits.is_null()) {
            let sum = r.digest() >> BUP_BLOBBITS;
            let rbits = BUP_BLOBBITS;
            while (rsum & 1) {
                rsum = rsum >> 1;
                rbits = rbits + 1;
            }
            unsafe {
                *bits = rbits;
            }
        }
        len + 1
    } else {
        0
    }
}


#ifndef BUP_NO_SELFTEST
#define BUP_SELFTEST_SIZE 100000

int bupsplit_selftest()
{
    uint8_t *buf = malloc(BUP_SELFTEST_SIZE);
    uint32_t sum1a, sum1b, sum2a, sum2b, sum3a, sum3b;
    unsigned count;
    
    srandom(1);
    for (count = 0; count < BUP_SELFTEST_SIZE; count++)
	buf[count] = random();
    
    sum1a = rollsum_sum(buf, 0, BUP_SELFTEST_SIZE);
    sum1b = rollsum_sum(buf, 1, BUP_SELFTEST_SIZE);
    sum2a = rollsum_sum(buf, BUP_SELFTEST_SIZE - BUP_WINDOWSIZE*5/2,
			BUP_SELFTEST_SIZE - BUP_WINDOWSIZE);
    sum2b = rollsum_sum(buf, 0, BUP_SELFTEST_SIZE - BUP_WINDOWSIZE);
    sum3a = rollsum_sum(buf, 0, BUP_WINDOWSIZE+3);
    sum3b = rollsum_sum(buf, 3, BUP_WINDOWSIZE+3);
    
    fprintf(stderr, "sum1a = 0x%08x\n", sum1a);
    fprintf(stderr, "sum1b = 0x%08x\n", sum1b);
    fprintf(stderr, "sum2a = 0x%08x\n", sum2a);
    fprintf(stderr, "sum2b = 0x%08x\n", sum2b);
    fprintf(stderr, "sum3a = 0x%08x\n", sum3a);
    fprintf(stderr, "sum3b = 0x%08x\n", sum3b);
    
    free(buf);
    return sum1a!=sum1b || sum2a!=sum2b || sum3a!=sum3b;
}

#endif // !BUP_NO_SELFTEST
