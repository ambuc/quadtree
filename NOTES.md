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

     ```
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     |  |  |  |  |  |  |  |  |  |  |       |           |                 |
     +--+--+--+--+--+--+--+--+--+--+       +     1     +        2        +
     |  |  |  |  |  |  |  |  |  |  |       |           |                 |
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     |  |  |  |  |xxxxxxxxxxx|  |  |       |           |xxxxxxxxxxx|     |
     +--+--+--+--+xxxxxxxxxxx+--+--+  -->  +           +xxxxxxxxxxx+     +
     |  |  |  |  |xxxxxxxxxxx|  |  |       |           |xxxx4.1xxxx| 4.2 |
     +--+--+--+--+xxxxxxxxxxx+--+--+       +           +xxxxxxxxxxx+     +
     |  |  |  |  |xxxxxxxxxxx|  |  |       |     3     |xxxxxxxxxxx|     |
     +--+--+--+--+--+--+--+--+--+--+       +           +--+--+--+--+--+--+
     |  |  |  |  |  |  |  |  |  |  |       |           |    4.3    | 4.4 |
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+

     ->

     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     |           |                 |       | 1.1 | 1.2 | 2.1 |    2.2    |
     +  1  +--+--+--+--+  2        +       +--+--+--+--+--+--+--+--+--+--+
     |     |\\\\\\\\\\\|           |       |     |\\\\\|\\\\\|           |
     |     |\\\\\\\\\\\|           |       | 1.3 |\1.4\|\2.3\|    2.4    |
     |     |\\\\\\\\\\\|           |       |     |\\\\\|\\\\\|           |
     +--+--+\\\\\\\\\\\+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     |     |\\\\\\\\\\\|/////|     |       |     |\\\\\|XXXXX|/////|     |
     |     |\\\\\\\\\\\|/////|     |       | 3.1 |\3.2\|4.1.1|4.1.2|     |
     |     |\\\\\\\\\\\|/////|     |       |     |\\\\\|XXXXX|/////|     |
     +     +--+--+--+--+/////+ 4.2 +  -->  +--+--+--+--+--+--+--+--+ 4.2 +
     |           |////4.1////|     |       |     |     |/////|/////|     |
     +           +///////////+     +       +     +     +4.1.3+4.1.4+     +
     |     3     |///////////|     |       | 3.3 | 3.4 |/////|/////|     |
     +           +--+--+--+--+--+--+       +     +     +--+--+--+--+--+--+
     |           |    4.3    | 4.4 |       |     |     |    4.3    | 4.4 |
     +--+--+--+--+--+--+--+--+--+--+       +--+--+--+--+--+--+--+--+--+--+
     TODO(ambuc): These are useful diagrams. What if I wrote a util to generate
     them for small quadtree sizes?

     ```

Rather than manage references and lifetimes up and down the tree, we 
 - assume a relatively low overlap cardinality (i.e. two, rarely three items
   overlapping)
 - assume a relatively small size per object s.t. O(IxJ) insert operations is
   bad but acceptable
 - what if you used hash? and expected that items implemented
   hash in such a way that mutations you perform to them do not affect their
   hash? (a constant uuid)
For the example above:
 - root
   [ 409 => Item { value: "\", region: (1,1)->2x2 }
   , 778 => Item { value: "/", region: (2,2)->2x2 }
   ]
   - 1
     * 1.4 #409
   - 2
     * 2.3 #409
   - 3
     * 3.2 #409
   - 4
     * 4.1 #778
       * 4.1.1 #409
     * 4.2
     * 4.3
     * 4.4

TODO(ambuc): I don't think Rust supports streaming mutable iterators. I might
have to implement something like the Entry API ?
