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

     1 =>
       1.1 => None
       1.2 => None
       1.3 => None
       1.4 => exact[ '\' ]
     2 =>
       2.1 => None
       2.2 => None
       2.3 => ref[ '\' ]
       2.4 => None
     3 =>
       3.1 => None
       3.2 => ref[ '\' ]
       3.3 => None
       3.4 => None
     4 =>
       4.1 =>
         exact[ '/' ]
         tree =>
           4.1.1 => ref [ '\' ]
           4.1.2 => None
           4.1.3 => None
           4.1.4 => None
       4.2 => None
       4.3 => None
       4.4 => None

     pub struct Quadtree<U, V> {
         depth: usize,
         region: Area<U>,

         values: Vec<(Area<U>, V)>, // Values contained within this node.
         exact_values: Vec<V>, // Values contained at exactly this region.
         ref_values: Vec<&V>,  // References to values this region also touches.

         subquadrants: [Option<Box<Quadtree<U, V>>>; 4],
         // - Can be None, if the qt isn't subdivided.
         // - Otherwise, is subdivided into 4 discrete area, but each could 
         //   point to no boxed qt all, if nothing is there.
         subquadrants: Option<[Option<Box<Quadtree<U,V>>>; 4]>,
     }

     ```
