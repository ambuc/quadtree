# Development notes / notes to myself:
 - Developing a Cargo library
   - Generating docs: `cargo doc --no-deps`
   - Previewing `README.md`: `grip README.md localhost:8000`.
 - Models for API
   - Conform to
     [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
     where possible
 - Notes on Quadtree performance
   - It's OK for insert/delete to be relatively expensive if that means query is
     cheap.
   - Right now we store each region at the lowest possible leaf which totally
     contains it. But we store multiple regions in a vector (bad), and filter
     through them at return time (bad).
   - TODO(ambuc): There should be a way to totally divide the grid around each
     inserted region.

     ```
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     |  |  |  |  |  |  |  |  |  |  |       |           |                 |
     +--+--+--+--+--+--+--+--+--+--+       +           +                 +
     |  |  |  |  |  |  |  |  |  |  |       |           |                 |
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     |  |  |  |  |xxxxxxxxxxx|  |  |       |           |xxxxxxxxxxx|     |
     +--+--+--+--+xxxxxxxxxxx+--+--+  -->  +           +xxxxxxxxxxx+     +
     |  |  |  |  |xxxxxxxxxxx|  |  |       |           |xxxxxxxxxxx|     |
     +--+--+--+--+xxxxxxxxxxx+--+--+       +           +xxxxxxxxxxx+     +
     |  |  |  |  |xxxxxxxxxxx|  |  |       |           |xxxxxxxxxxx|     |
     +--+--+--+--+--+--+--+--+--+--+       +           +--+--+--+--+--+--+
     |  |  |  |  |  |  |  |  |  |  |       |           |           |     |
     +--+--+--+--+--+--+--+--+--+--+       +           +           +     +
     |  |  |  |  |  |  |  |  |  |  |       |           |           |     |
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+

     ```
