# RainbowTable

A command line tool to aid in the creation of rainbow tables.

## FAQ

### What's a rainbow table?

Here, I turn to my good friend Wikipedia:

```text
A rainbow table is a precomputed table for caching the outputs of a cryptographic hash function[...].
```

In layman's terms, a rainbow table is a database that pairs up a plaintext string with its hash equivalents.

### What's a hash function?

Once again, Wikipedia:

```text
A hash function is any function that can be used to map data of arbitrary size to fixed-size values, though there are some hash functions that support variable length output.

```text
And once again, in layman's terms, a hash function takes data of any size and converts it to a fixed size. Since data is most likely either lost or padded in some way, hash functions relied upon to be **one way and irreversible**. A rainbow table aims to fix that by letting you look up a hash and get its plaintext value.

### Cool, how do I use this?

Here's how you install and use it:

```text
cargo install rainbowtable
rainbowtable
```

At that point it should give you a list of options -  welcome to the club!

### Aren't passwords stored as hashes? Doesn't this mean that if you get your hands on someone's hashed password, you can get it in plaintext?

Yes, and this brings me to my next point:

## DISCLAIMER

This project is to be used for educational purposes only. I (Astro Orbis, Intragon) am not responsible for any malicious activity done with this tool. (Besides, there are way better things you can use if you're looking to get up to some shenanigans.)

### Sponsor me

If you really liked this project, or you want me to make a similar one, take a look at my [ko-fi](https://ko-fi.com/astroorbis)!
