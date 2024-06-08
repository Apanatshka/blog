+++
title = "Learn Rust by project"
date = "2018-04-30"
aliases = ["compsci/2018/04/30/learn-rust-by-project"]
taxonomies.tags = ["rust"]
+++

More than a year ago a friend of mine wanted to learn a bit more about Rust by trying out a project. He had a nice project in mind which suits Rust quite well I think. For fun I joined his effort and created [an implementation](https://gitlab.com/Apanatshka/cobs) at the same time as he did, discussing and comparing along the way. In this post I'll tell you about the project specifics, but the point of the post is more an encouragement. If you've read about Rust before but haven't tried it yet, find a small project like the one below, and learn Rust in a fun and hands-on way yourself. It's a great programming language, I highly recommend it. 

# The project: consistent overhead byte stuffing

Computer networking can be messy business. Depending on what [layer of the network](https://en.wikipedia.org/wiki/OSI_model#Description_of_OSI_layers) your software is operating in, you need to worry about different kinds of errors. If you're receiving raw bytes, you might run into the issue of corrupted messages. You could throw away such a message entirely... Or you could try to chop it into chunks with a clear boundary and recover at the next boundary. 

Byte stuffing is the process of stuffing bytes into a smaller range of values than the full byte, so you can use the unused values for something special like the boundaries of messages. The usual terminology is splitting your bytes of data into _frames_ and using _sentinel values_ to delimit the frames. 

The problem that byte stuffing solves then, is what to do with values in your data that are the sentinel value you picked for delimiting frames. These should be turned into something else that can be reliably decoded again. The project of this post is to implement an algorithm for Consistent Overhead Byte Stuffing, or COBS. This algorithm has an overhead of at least one byte and at most one byte in 254 rounded up. If I'd been presented with this problem myself before I'd heard of COBS, I would have probably done something like pick two byte values, one delimiter and one "escape character". The worst case for that is something like twice the size of the message. So this algorithm is pretty cool. I'll explain it in my own words, but the [Wikipedia article](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing) is very nice too, so browse that if you don't follow everything here. 

# COBS in short

Let's choose zero as our sentinel values, our _frame marker_. If we need to recover from some error in the middle of the stream of frames, we just look for the next zero, that's the end of a frame. Then the first byte is our consistent minimum overhead byte, which starts the COBS encoding. This byte tells us the offset to where the next zero should have been in the message. Until that offset is reached, the bytes should be the original message. If in those bytes you find a zero, the message is definitely corrupted and you should skip to the next frame. Once you reach the offset, instead of a zero you should find another number, which is the offset from there to the next zero in the original message. So each zero is turned into a higher number of where the next zero is. The last zero points to the place where the zero of the end of the frame should be[^overhead].

Let's call these offsets to a zero _zero markers_. The first _zero marker_ is _fake_, since it doesn't mark a zero at its place. We need it to point out the first actual zero. There can be more fake zero markers in the message, and this is where the worst case overhead comes from: what if two zeroes are further apart than the size of a byte? Assuming we're speaking of [octets](https://en.wikipedia.org/wiki/Octet_(computing)), which is usually the case these days, we have 0-255 as the normal value range. We're changing that range to 1-255. So if 255 is the maximum value for our zero marker, we cannot have more than 254 consecutive non-zero bytes. To fix this we say that zero markers with value 255 signify that the _next_ zero marker is _fake_. Again, a fake zero marker does not signify a zero, but just how many bytes to read until we reach the next zero marker. 

# COBS in Rust

Now that we've seen a prose description of COBS in Rust, let's implement an encode and a decode function for COBS. In this case I'll present an implementation that is uses a sentinel value of zero, adds that zero as part of the encode procedure, and expects it during decode. Note that this is not going to be the most beautiful implementation possible. We're not using traits from Rust's standard library, such as `Read` and `Write` even though these work with bytes. We're hard-coding the sentinel value to `0`. 

## Tests

To start things off, let's define the types for encode and decode, and then write some tests to make our understanding of the algorithm executable. We start out with some unit tests, and some property based tests using [quickcheck](https://crates.io/crates/quickcheck). Property based tests use a function from some input to boolean and given that input check if a property holds. 

With the property based tests we check that encoding and decoding a given vector of bytes comes to the same thing. The quickcheck framework then generates some random vectors of bytes and checks if our property holds. If it doesn't, the framework shrinks the counter example with some heuristics. It's pretty cool stuff. I recommend using this form of testing whenever you can. 

```rust
//! Consistent overhead byte stuffing
//! =================================
//!
//! This encoding allows for packet loss in a stream of bytes by dividing data into frames.
//!
//! 0 = framemarker, the thing you search for when you recover in the middle of a stream. It marks
//!  the end of a frame.
//! Zeromarkers both mark a zero and have a value of where the next zeromarker is. There are also
//! special zeromarkers, which say the next zeromarker is fake. Fake zeromarkers don't mark zeroes,
//! they only tell where the next zeromarker is.
//! The first byte of a frame is a fake zeromarker.
//! Other bytes are normal bytes.
//! When normally (in the original data) the next zero occurs, this is another zeromarker. The
//!  first (fake) zeromarker will have the offset after which the next zeromarker occurs.
//! The special zeromarker 255 predicts that the next zeromarker is fake. The reason for calling it
//! fake is to support data where the are more than 255 bytes between zeroes.

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

use std::iter;

pub fn encode(data: &[u8], encoded_data_buffer: &mut Vec<u8>) -> usize {
    unimplemented!()
}

pub fn decode(encoded_data: &[u8], decoded_data_buffer: &mut Vec<u8>) -> Result<usize, usize> {
    unimplemented!()
}

pub fn max_encoded_size(input_size: usize) -> usize {
    input_size * (u8::max_value() as f32 / (u8::max_value() - 1) as f32).ceil() as usize + 1
}

pub fn max_decoded_size(encoded_size: usize) -> usize {
    encoded_size - 2
}

#[cfg(test)]
mod tests {
    macro_rules! unit_test_set {
        ($assert:path) => {
            #[test]
            fn empty() {
                $assert(&[], &[1, 0]);
            }

            #[test]
            fn zero() {
                $assert(&[0], &[1, 1, 0]);
            }

            #[test]
            fn one() {
                $assert(&[1], &[2, 1, 0]);
            }

            #[test]
            fn byte_max() {
                $assert(&[255], &[2, 255, 0]);
            }

            #[test]
            fn five_zeroes() {
                $assert(&[0; 5], &[1, 1, 1, 1, 1, 1, 0]);
            }

            #[test]
            fn five_ones() {
                $assert(&[1; 5], &[6, 1, 1, 1, 1, 1, 0]);
            }

            #[test]
            fn byte_max_zeroes() {
                let mut output = Vec::with_capacity(257);
                output.extend_from_slice(&[1; 256]);
                output.push(0);
                $assert(&[0; 255], output.as_slice());
            }

            #[test]
            fn byte_max_ones() {
                let mut output = Vec::with_capacity(258);
                output.push(255);
                output.extend_from_slice(&[1;254]);
                output.extend_from_slice(&[2, 1, 0]);
                $assert(&[1; 255], output.as_slice());
            }
        }
    }

    mod encode {
        use super::super::encode;

        fn check_encoded_vs_given(data: &[u8], encoded: &[u8]) {
            let mut buffer = Vec::new();
            let _ = encode(data, &mut buffer);
            assert_eq!(encoded, buffer.as_slice());
        }

        unit_test_set!(check_encoded_vs_given);
    }

    mod decode {
        use super::super::decode;

        fn check_decoded_vs_given(data: &[u8], encoded: &[u8]) {
            let mut buffer = Vec::new();
            let _ = decode(encoded, &mut buffer);
            assert_eq!(data, buffer.as_slice());
        }

        unit_test_set!(check_decoded_vs_given);
    }

    use super::{encode, decode};

    quickcheck! {
        fn encode_decode_identity(data: Vec<u8>) -> bool {
            let mut b1 = Vec::new();
            let mut b2 = Vec::new();
            let _ = encode(data.as_slice(), &mut b1);
            let _ = decode(b1.as_slice(), &mut b2);
            data == b2
        }
    }
}
```

## Constants

I really dislike magic values in my code. Here are some constants we'll use: 

```rust
const MAXu8: u8 = u8::max_value();
const MAX: usize = MAXu8 as usize;
const MAX_CONSECUTIVE: usize = MAX - 1;
```

The `MAX*` values are just shorter without using `255` literally. I did use them in the tests because there are related numbers off by 2 or 3. It made more sense in the unit tests, which encode specific examples. The code is abstract, should work for any case. 

## Decode

So let's decode some bytes. For every byte we can either interpret it as a zero marker if it's at the right offset, or we can copy the byte to the output verbatim. If the copied byte is zero, that's an error. Unless it's a predicted zero marker, in which case we're successfully finished. If it's a zero marker that's not fake, we push a zero instead of the byte value of the marker. We also need to compute the index of the next zero marker with the relative offset. This boils down to the following code:

```rust
pub fn decode(encoded_data: &[u8],
              decoded_data_buffer: &mut Vec<u8>)
              -> Result<usize, usize> {
    let mut zero_marker_index: usize = 0;
    let mut fake = true;
    let mut written = 0;
    for (index, &byte) in encoded_data.iter().enumerate() {
        if index == zero_marker_index {
            if byte == 0 {
                return Ok(written); // framemarker, we're done
            }
            if !fake {
                decoded_data_buffer.push(0);
                written += 1;
            }
            zero_marker_index = index + byte as usize;
            fake = byte == MAXu8;
        } else {
            if byte == 0 {
                break; // fail
            }
            decoded_data_buffer.push(byte);
            written += 1;
        }
    }
    Err(written) // fail
}
```

The `zero_marker_index` is the absolute offset from the start of the `encoded_data` slice. The boolean `fake` is for remembering if that zero marker will be fake or not. We also track how many bytes we've `written` in the buffer of decoded data. We break from the loop to fail, because we may also find no `0` at all and run out of encoded data, which is also a corner case where we should fail. 

## Encode (buffer input)

There are two different ways you can implement COBS. One is to look ahead in the data for the zero. This means to need a buffer of 254 bytes at most, but you can sequentially output the encoded bytes. Is to sequentially read the input byte without buffering them, instead buffering the output so you can go back and fill in the space you reserved for the zero marker once you've seen the next zero. Let's first look at the look-ahead version:

```rust
pub fn encode_lookahead(data: &[u8], encoded_data_buffer: &mut Vec<u8>) -> usize {
    let start_length_out_buffer = encoded_data_buffer.len();

    let mut data_iter = data.iter().chain(iter::once(&0)).peekable();
    let mut buf = [0_u8; MAX_CONSECUTIVE];
    let mut buf_index: usize;

    while let Some(_) = data_iter.peek() {
        buf_index = 0;

        // Find the next zero, copy bytes seen into buffer
        for &byte in data_iter.by_ref().take_while(|&&b| b != 0) {
            buf[buf_index] = byte;
            buf_index += 1;
            debug_assert!(buf_index <= buf.len());
            if buf_index == buf.len() {
                break;
            }
        }

        // Write where next zero is, then write the data from the buffer
        // Note the +1, since buf_index starts at zero and the next zero is always at least one away
        encoded_data_buffer.push(buf_index as u8 + 1);
        encoded_data_buffer.extend_from_slice(&buf[0..buf_index]);
    }

    encoded_data_buffer.push(0);

    encoded_data_buffer.len() - start_length_out_buffer
}
```

Given that we take in some `Vec<u8>` that we append to, we should save the length and return the difference in length as the amount of bytes written. Then we create an iterator over the data, followed by an extra zero, which should be `peekable`. This means we can look ahead without consuming to see if the iterator is done yet. While it isn't done, we set keep an index of the used part of the buffer. For the bytes in the iterator, we just add them to the buffer unless it's a zero or the buffer is full. Then we write the zero marker first using the buffer index to see how many non-zeroes we found ahead. And we copy over the buffer. The extra zero on the data iterator compensates for the extra zero-marker at the front of the message. The extra zero pushed at the end ends the message. 

## Encode (buffer output)

If we buffer the output, we can just write a bogus value for the zero marker, remember its index, and overwrite it later. Again only 255 bytes need to be buffered at a maximum, although this isn't visible in our implementation. 

```rust
pub fn encode(data: &[u8], encoded_data_buffer: &mut Vec<u8>) -> usize {
    let start_length_out_buffer = encoded_data_buffer.len();

    // Note that we always start from 1, so we count MAX_CONSEQUTIVE bytes of non-zero data
    let mut non_zero_count = 1_usize;
    let mut zero_marker_index;

    macro_rules! next_zero_marker { () => {
        encoded_data_buffer[zero_marker_index] = non_zero_count as u8;
        non_zero_count = 1_usize;
        zero_marker_index = encoded_data_buffer.len();
        encoded_data_buffer.push(0);
    }};
    zero_marker_index = encoded_data_buffer.len();
    encoded_data_buffer.push(0);

    // NOTE: the extra zero at the end will become the framemarker
    for &byte in data.iter().chain(iter::once(&0)) {
        if byte == 0 {
            next_zero_marker!();
        } else {
            encoded_data_buffer.push(byte);
            non_zero_count += 1;
            debug_assert!(non_zero_count <= MAX);
            if non_zero_count == MAX {
                next_zero_marker!();
            }
        }
    }

    encoded_data_buffer.len() - start_length_out_buffer
}
```

In the end, I don't think the memory requirements and timing behaviour of the two different options should be very different. But to put that to the test, I've written a little benchmark:

```rust
macro_rules! gen_benches {
    ($prefix:ident) => {
        mod $prefix {
            use test::Bencher;
            use cobs::$prefix;
            use super::LOREM_IPSUM_RAW;

            #[bench]
            fn encode_r1(b: &mut Bencher) {
                b.iter(|| {
                    let mut lorem_ipsum_encoded = Vec::new();
                    let _ = $prefix(LOREM_IPSUM_RAW, &mut lorem_ipsum_encoded);
                });
            }
        }
    }
}

gen_benches!(encode);
gen_benches!(encode_lookahead);
```

This uses the first 4 paragraphs or so from [Lorem Ipsum](https://lipsum.com/), and on my machine the input buffering version is always faster:

```
 name       encode:: ns/iter  encode_lookahead:: ns/iter  diff ns/iter   diff %  speedup 
 encode_r1  4,243             3,745                               -498  -11.74%   x 1.13
```

## Faster decode

Say we wanted to seriously speed up our decoding. We could do so by dropping the check of an unexpected zero. Why? Well if you drop that check, a zero marker will tell you exactly how many bytes you can copy over verbatim before the next marker. Which you can do in Rust with the `extend_from_slice` function, which is probably a bit faster than a manual loop. Let's try that out:

```rust
pub fn decode(encoded_data: &[u8], decoded_data_buffer: &mut Vec<u8>) -> Result<usize, usize> {
    if encoded_data.len() == 0 {
        return Err(0)
    }

    let start_length_out_buffer = decoded_data_buffer.len();
    let mut index = 0;
    let mut zero_marker = encoded_data[index] as usize;

    loop {
        let next_index = index + zero_marker;
        if zero_marker == 0 {
            return Ok(decoded_data_buffer.len() - start_length_out_buffer);
        }
        if next_index >= encoded_data.len() {
            return Err(decoded_data_buffer.len() - start_length_out_buffer)
        }

        decoded_data_buffer.extend_from_slice(&encoded_data[index+1..next_index]);

        if zero_marker != u8::max_value() as usize && encoded_data[next_index] != 0 {
            decoded_data_buffer.push(0);
        }

        zero_marker = encoded_data[next_index] as usize;
        index = next_index;
    }
}
```

We don't use iterators any more in this code. This is not very idiomatic for Rust, since Rust can more easily eliminate bounds checks for loops over iterators. However, we do get the `extend_from_slice` which is hopefully more efficient. So what we do is keep the index into the data around, and look up the zero marker. If the zero marker is zero, we're done, end of message. If the index is out of bounds, that's an error. Otherwise we extend from the `index+1` up to (not including) the next index. The `+1` is because `index` always points to a zeromarker. Should the zero marker be `255` or the next zero marker be `0`, then we don't need to add a zero after the copied data. Then we update the zero marker and index. 

To test this we run another benchmark, this time decoding the encoded lorem ipsum text. The results are quite promising:

```
 name       naive_decode:: ns/iter  decode:: ns/iter  diff ns/iter   diff %  speedup 
 decode_r1  3,962                   361                     -3,601  -90.89%  x 10.98
```

Naturally, this faster decode _is_ too permissive. So a quickcheck test such as the following will fail most of the time by finding an incorrect COBS encoded message with an unexpected zero.

```rust
    quickcheck! {
        fn naive_decode_eq_decode(data: Vec<u8>) -> bool {
            let mut b1 = Vec::new();
            let mut b2 = Vec::new();
            let r1 = decode(data.as_slice(), &mut b1);
            let r2 = naive_decode(data.as_slice(), &mut b2);
            if let (Err(_), Err(_)) = (r1,r2) {
                true
            } else {
                r1 == r2 && b1 == b2
            }
        }
    }
```

## Faster encode

Perhaps we can also improve our look-ahead encoding, by not explicitly buffering anything. If instead we just find the position of the next zero, we can use some index juggling:

```rust
pub fn encode_itertools(data: &[u8], encoded_data_buffer: &mut Vec<u8>) -> usize {
    use itertools::Itertools;

    let start_length_out_buffer = encoded_data_buffer.len();
    let mut index = 0_usize;
    for z_index in data.iter().chain(iter::once(&0)).positions(|&b| b == 0) {
        debug_assert!(z_index >= index);
        // index is always still-unvisited, so when z_index == index, we need to write a 1
        macro_rules! offset_between_zeroes { () => {z_index - index + 1}}
        while offset_between_zeroes!() >= MAX {
            encoded_data_buffer.push(MAXu8);
            encoded_data_buffer.extend_from_slice(&data[index..index + MAX_CONSECUTIVE]);
            index += MAX_CONSECUTIVE as usize;
        }
        encoded_data_buffer.push(offset_between_zeroes!() as u8);
        encoded_data_buffer.extend_from_slice(&data[index..z_index]);
        index = z_index + 1;
    }
    encoded_data_buffer.push(0);

    encoded_data_buffer.len() - start_length_out_buffer
}
```

We use [itertools](https://crates.io/crates/itertools) here for the `positions` iterator method. We go over the whole data looking for the indices of the zero bytes. We've encoded up to `index`, so if the offset between there and the zero is too far, we need to add the special zero markers and the `MAX_CONSECUTIVE` number of data bytes. In the end we always write the offset as zero marker, then the data up to the zero, then set the index to _after_ the zero.

This is again a bit faster because we don't copy each byte into a buffer only to copy the buffer again. The positions iterator method now does the look-ahead for us. 

```
 name       encode_lookahead:: ns/iter  encode_itertools:: ns/iter  diff ns/iter   diff %  speedup 
 encode_r1  3,750                       2,756                               -994  -26.51%   x 1.36
```

# Conclusion

So we've seen some Rust code today that was hopefully readable to you. Tests are easy because they're built-in. Property based tests are just a crate import away. Benchmarks require the nightly compiler, but only those do, so you can just use `cargo +nightly bench` to run them. The comparison tables are generated with [`cargo-benchcmp`](https://crates.io/crates/cargo-benchcmp). 

I've shown some of my implementations for COBS in Rust, but this was only a learning exercise. I hope this inspires you to find a project of your own to get more experience with Rust. A _real_ implementation of COBS in Rust can be found in the [`cobs`](https://crates.io/crates/cobs) crate, which allows you to use whichever sentinel value you want, can decode in-place, and doesn't even use vectors so you can use the crate without the standard library. 


[^overhead]: If you count the zero at the end of a frame as part of the COBS algorithm, it has a minimum offset of 2. But apparently people usually count that as a separate "packetize" step, or so it says on Wikipedia. 
