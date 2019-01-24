# quadtree-impl
General purpose [point/region Quadtree](https://en.wikipedia.org/wiki/Quadtree)
implementation for Rust.

## Notes to myself:
 - Developing a Cargo library
   - Generating docs: `cargo doc --no-deps`
   - Previewing `README.md`: `grip README.md localhost:8000`.
 - Models for API
   - Conform to
     [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
     where possible
