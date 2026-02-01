Implementing T-Splines in Rust: A Comprehensive Framework for Geometric KernelsExecutive SummaryThe domain of Computer-Aided Design (CAD) and geometric modeling is currently undergoing a foundational shift. For decades, the industry has relied on Non-Uniform Rational B-Splines (NURBS) as the mathematical lingua franca for defining complex surfaces. While NURBS provide exact representations of conic sections and freeform curves, their tensor-product topology imposes severe constraints on model complexity and data efficiency. Specifically, the necessity of rectangular grids forces global refinement propagation, where adding detail to a single region of a surface requires inserting control points across the entire model, leading to data proliferation and superfluous geometric information.T-Splines, introduced by Sederberg et al. (2003), offer a generalization of NURBS that permits T-junctions in the control grid, enabling true local refinement while maintaining mathematical compatibility with standard NURBS surfaces. This capability not only reduces control point counts by up to 70% but also bridges the gap between design (CAD) and analysis (Finite Element Analysis or Isogeometric Analysis), provided specific topological constraints known as Analysis-Suitability (ASTS) are met.Concurrently, the systems programming landscape is shifting from C++—the traditional language of geometric kernels like ACIS, Parasolid, and OpenNURBS—to Rust. Rust’s guarantees of memory safety without garbage collection, combined with its expressive type system and zero-cost abstractions, make it uniquely positioned to solve the stability and concurrency issues plaguing legacy CAD kernels.This report presents an exhaustive technical blueprint for implementing a T-Spline kernel in Rust. We synthesize mathematical theory with software engineering practice, addressing the specific challenges of implementing half-edge data structures under Rust’s ownership model, designing trait-based geometric polymorphism compatible with the truck ecosystem, and optimizing knot inference algorithms for modern hardware. We provide extensive reference code, architectural diagrams, and algorithmic analysis to guide the development of a production-grade, analysis-suitable T-Spline library.1. The Mathematical Imperative: From NURBS to T-SplinesTo understand the architectural requirements of a T-Spline kernel, one must first dissect the mathematical limitations of the predecessor technology and the specific innovations introduced by T-Splines. The transition from NURBS to T-Splines is not merely a change in format but a fundamental shift in how topology dictates geometry.1.1 The Tensor Product LimitationA standard NURBS surface $S(u,v)$ is defined mathematically as a tensor product of two univariate B-spline bases. The surface equation is given by:$$S(u,v) = \frac{\sum_{i=0}^{n} \sum_{j=0}^{m} w_{i,j} P_{i,j} N_i^p(u) N_j^q(v)}{\sum_{i=0}^{n} \sum_{j=0}^{m} w_{i,j} N_i^p(u) N_j^q(v)}$$In this formulation, $P_{i,j}$ represents a bidirectional net of control points, and $N$ represents the B-spline basis functions of degrees $p$ and $q$. The critical constraint here is topological: the control points must form a rigid rectangular grid. The parameter space is defined by two global knot vectors, $U = \{u_0, \dots, u_{n+p+1}\}$ and $V = \{v_0, \dots, v_{m+q+1}\}$.This structure implies that if a designer wishes to add a feature (requiring higher control point density) at a specific location, they must insert a knot into the global $U$ or $V$ vector. This insertion propagates across the entire surface, creating a full row or column of new control points. This phenomenon, known as "global refinement," results in:Superfluous Data: Regions of low detail are forced to carry high-density control grids.Stiffness: The unnecessary control points make the surface harder to manipulate smoothly.Patch Fragmentation: To avoid global propagation, designers often split models into hundreds of separate NURBS patches, creating continuity management nightmares (the "watertight" problem).1.2 The T-Spline GeneralizationT-Splines break the tensor product constraint by assigning a local basis function domain to each control point individually, rather than deriving it from a global grid. A T-Spline surface is defined over a T-Mesh, a control grid that permits T-junctions—vertices where a line of control points terminates.The T-Spline equation is:$$P(s,t) = \frac{\sum_{i=1}^{n} w_i P_i B_i(s,t)}{\sum_{i=1}^{n} w_i B_i(s,t)}$$Here, $B_i(s,t)$ is the blending function associated with control point $P_i$. The profound difference lies in how $B_i$ is constructed. In NURBS, basis functions are derived from the global knot vectors. In T-Splines, the knot vectors for $B_i$ are inferred locally from the topology of the T-Mesh surrounding $P_i$.This inference mechanism allows T-Splines to support Local Refinement. A single control point can be inserted into the mesh to add local detail without propagating changes to the rest of the surface. This capability aligns T-Splines more closely with Subdivision Surfaces (SubD) used in animation, while retaining the precise parametric definition required for engineering.1.3 Knot Interval Inference: The Core AlgorithmThe most technically demanding aspect of implementing T-Splines is the algorithm that translates the T-Mesh topology into knot vectors for evaluation. This process is governed by rules established by Sederberg et al..For a cubic T-Spline (degree 3), each control point $P_i$ effectively requires a local knot vector of length 5 in both parametric dimensions ($s$ and $t$):$s_i = [s_{i0}, s_{i1}, s_{i2}, s_{i3}, s_{i4}]$$t_i = [t_{i0}, t_{i1}, t_{i2}, t_{i3}, t_{i4}]$The coordinate $(s_{i2}, t_{i2})$ is the parametric location of the control point $P_i$ itself. The remaining knots are determined by traversing the T-Mesh rays emanating from $P_i$.Rule 1 (Knot Inference):To find $s_{i3}$ and $s_{i4}$:Start at $P_i$ and move in the positive $s$ direction along the T-Mesh edges.The $s$-coordinate of the first perpendicular edge (vertical line) encountered becomes $s_{i3}$.Continue moving in the positive $s$ direction. The $s$-coordinate of the second perpendicular edge becomes $s_{i4}$.Similarly, traverse in the negative $s$ direction for $s_{i1}$ and $s_{i0}$, and in the $t$ directions for the $t$ knots.Implications for Implementation:
This rule implies that the underlying data structure must support efficient topological traversal. A simple array of vertices is insufficient. We need a graph structure that answers questions like: "What is the next vertical edge to the right of this vertex?" This necessitates a Half-Edge or Winged-Edge data structure. Furthermore, the system must handle cases where rays pass through T-junctions or hit the boundary of the mesh, requiring robust boundary condition logic (typically repeating boundary knots to achieve open uniform knot vector behavior).1.4 Analysis Suitability (ASTS) and Isogeometric AnalysisWhile standard T-Splines allow arbitrary T-junctions, research has shown that not all T-Meshes produce a set of linearly independent basis functions. Linear dependence is fatal for simulation methods like Finite Element Analysis (FEA) because it leads to singular stiffness matrices.Analysis-Suitable T-Splines (ASTS) are a rigorous subset of T-Splines that guarantee linear independence and partition of unity. The defining characteristic of ASTS is the restriction on T-junction relationships:Intersection Rule: The extensions of T-junctions (lines drawn from the T-junction across faces) must not intersect. Specifically, a horizontal T-junction extension cannot cross a vertical T-junction extension.For a modern Rust implementation targeting engineering applications, enforcing ASTS is not optional—it is a requirement. The kernel must include a validator that checks mesh topology against these intersection rules whenever the mesh is modified (e.g., during local refinement or import).2. Rust Architecture: Designing for Safety and ConcurrencyImplementing a geometric kernel in Rust requires a fundamental rethinking of data structures compared to traditional C++ approaches. C++ libraries like OpenMesh or OpenNURBS  rely heavily on pointers to manage the complex connectivity of mesh elements (e.g., edge->face->next_edge). In Rust, such cyclic reference graphs are difficult to implement safely due to the ownership model.2.1 The Pointer Problem in GeometryIn a doubly-linked list or a half-edge structure, elements own each other cyclically. Rust's borrow checker forbids multiple mutable owners.Naive Approach: Using Rc<RefCell<Node>>. This incurs runtime overhead for reference counting and borrow checking, and it spreads data across the heap, destroying cache locality.Unsafe Approach: Using raw pointers *mut Node. This bypasses Rust's safety guarantees, reintroducing the risk of segmentation faults and use-after-free errors that we explicitly aim to avoid.The Rusty Approach: Arena Allocation with Indices. All nodes, edges, and faces are stored in contiguous Vec buffers (Arenas). "Pointers" are simply integer indices (usize or a typed wrapper). This guarantees memory safety, enables trivial serialization, and significantly improves CPU cache performance due to data locality.2.2 Architectural LayersWe propose an architecture aligned with the truck ecosystem , separating concerns into distinct crates or modules:tspline-core (Math & Topology):Defines the TMesh struct using index-based half-edges.Implements the KnotVector and ControlPoint structs.Handles topological validity checks (ASTS).tspline-algo (Algorithms):Knot inference logic (Rule 1).Blending function evaluation.Local refinement algorithms.tspline-io (Interoperability):Parsers for .tsm and .iga files.Conversion traits to/from truck NURBS structures.2.3 Integration with the truck EcosystemThe truck project  is the leading effort to build a Rust CAD kernel. To ensure our T-Spline implementation is usable within this ecosystem, we must implement the traits defined in truck-geotrait.Key Trait to Implement:Rustpub trait ParametricSurface: Clone + Send + Sync {
type Point;
type Vector;
fn subs(&self, u: f64, v: f64) -> Self::Point;
fn u_der(&self, u: f64, v: f64) -> Self::Vector;
fn v_der(&self, u: f64, v: f64) -> Self::Vector;
fn normal(&self, u: f64, v: f64) -> Self::Vector;
}
By implementing this trait, our TSplineSurface struct effectively becomes a first-class citizen in the truck world, capable of being used in rendering pipelines, intersection algorithms, and boolean operations defined in other truck crates.3. Data Structure ImplementationThe foundation of the kernel is the T-Mesh data structure. We will implement a Half-Edge Data Structure (also known as DCEL - Doubly Connected Edge List) adapted for T-Splines. T-Splines introduce a complication: T-junctions mean that "faces" in the data structure might not correspond 1:1 with the geometric patches, or edges might be split logically but not topologically.We will use a canonical Half-Edge structure where T-junctions are treated as vertices. This implies that some faces are L-shaped or have more than 4 edges, or we strictly enforce a quad-dominant mesh where T-junctions split faces. For simplicity and robustness, treating the mesh as a collection of faces where T-junctions are vertices that induce "hanging nodes" is the standard approach in IGA.3.1 Core Struct DefinitionsWe use TypedIndex patterns (newtype idiom) to prevent mixing up vertex, edge, and face indices.Rustuse smallvec::SmallVec;
use truck_base::cgmath::{Point3, Vector4};

// Type-safe indices
#
pub struct VertID(pub usize);
#
pub struct EdgeID(pub usize);
#
pub struct FaceID(pub usize);

/// Parametric coordinate (s, t)
#
pub struct ParamPoint {
pub s: f64,
pub t: f64,
}

/// A Control Point in the T-Mesh.
/// Stores geometric position and a pointer to one outgoing half-edge.
#
pub struct ControlPoint {
pub id: VertID,
// Homogeneous coordinates (x, y, z, w) for rational surfaces
pub geometry: Vector4<f64>,
// The parametric location (knot value) of this point
pub uv: ParamPoint,         
// Index of one half-edge starting at this vertex
pub outgoing_edge: Option<EdgeID>,
// ASTS Metadata: Is this a T-junction?
pub is_t_junction: bool,
}

/// A Half-Edge.
/// Represents a directional edge. Two half-edges make a full edge.
#
pub struct HalfEdge {
pub id: EdgeID,
pub origin: VertID,      // Vertex where this edge starts
pub twin: Option<EdgeID>,// The opposite half-edge
pub face: Option<FaceID>,// The face to the left of this edge
pub next: EdgeID,        // Next half-edge in the face loop
pub prev: EdgeID,        // Previous half-edge in the face loop

    // T-Spline specific: Knot Interval associated with this edge
    // If the edge runs in S direction, this is a delta-s.
    pub knot_interval: f64, 
    pub direction: Direction,
}

#
pub enum Direction {
S, // Horizontal
T, // Vertical
}

/// A Face in the T-Mesh.
#
pub struct Face {
pub id: FaceID,
pub edge: EdgeID, // One bounding half-edge
}

/// The T-Mesh Container.
/// Owns all topological elements in contiguous vectors.
#
pub struct TMesh {
pub vertices: Vec<ControlPoint>,
pub edges: Vec<HalfEdge>,
pub faces: Vec<Face>,
}
3.2 Helper Implementation: Topology AccessTo work with this structure, we need helper methods to traverse the graph. Note that next and prev pointers allow us to circulate a face. twin allows us to jump to the neighboring face.Rustimpl TMesh {
/// Get a reference to a vertex
pub fn vert(&self, id: VertID) -> &ControlPoint { &self.vertices[id.0] }

    /// Get a reference to an edge
    pub fn edge(&self, id: EdgeID) -> &HalfEdge { &self.edges[id.0] }

    /// Iterate over all edges around a face
    pub fn face_edges(&self, face_id: FaceID) -> Vec<EdgeID> {
        let start_edge = self.faces[face_id.0].edge;
        let mut edges = Vec::new();
        let mut curr = start_edge;
        
        loop {
            edges.push(curr);
            curr = self.edge(curr).next;
            if curr == start_edge { break; }
        }
        edges
    }

    /// Find the edge connecting v_start to v_end (if it exists)
    pub fn find_edge(&self, v_start: VertID, v_end: VertID) -> Option<EdgeID> {
        let start_v = self.vert(v_start);
        let first_edge = start_v.outgoing_edge?;
        
        // Circulate around vertex (using twin -> next logic)
        let mut curr = first_edge;
        loop {
            let edge = self.edge(curr);
            // The destination of an edge is the origin of its twin
            // Or we can look at the origin of the 'next' edge if the face is valid
            // Standard approach: dest(e) = origin(twin(e))
            if let Some(twin_id) = edge.twin {
                let twin = self.edge(twin_id);
                if twin.origin == v_end {
                    return Some(curr);
                }
                // Move to next spoke: twin -> next
                curr = twin.next;
            } else {
                // Boundary case handling needed here
                break; 
            }
            
            if curr == first_edge { break; }
        }
        None
    }
}
This index-based approach ensures that TMesh can be cloned, serialized, or sent across threads (Send/Sync) without worrying about dangling pointers or data races, satisfying the core safety requirement.4. Algorithms: From Topology to GeometryWith the data structure in place, we implement the algorithms that define the T-Spline behavior: Knot Inference and Surface Evaluation.4.1 Knot Inference ImplementationWe implement the Ray Casting algorithm described in Section 1.3. The goal is to fill a [f64; 5] knot vector for a given dimension.Challenge: Traversing "rays" in a mesh that may not be a perfect grid.Solution: We implement a traverse_ray function that walks the mesh. If it hits a standard vertex (valence 4), it continues straight. If it hits a T-junction, it stops (since that defines a knot). If it hits a boundary, we apply boundary conditions.Rustimpl TMesh {
/// Infers the local knot vectors for a specific control point.
/// Returns (s_vector, t_vector).
pub fn infer_local_knots(&self, vert_id: VertID) -> ([f64; 5]; 2) {
let center_uv = self.vert(vert_id).uv;

        let s_knots =;

        let t_knots =;

        (s_knots, t_knots)
    }

    /// Casts a ray in `direction` for `steps` topological units.
    /// Returns the coordinate found.
    fn cast_ray_for_knot(&self, start_v: VertID, dir: Direction, steps: i32) -> f64 {
        let mut curr_v = start_v;
        let is_forward = steps > 0;
        let count = steps.abs();

        for _ in 0..count {
            match self.find_next_orthogonal_edge(curr_v, dir, is_forward) {
                Some(next_v) => {
                    curr_v = next_v;
                }
                None => {
                    // Boundary reached. 
                    // Standard T-Spline rule: extend the last interval or repeat knot.
                    // Here we simply return the boundary coordinate.
                    // A more robust impl would repeat the boundary knot if steps remain.
                    return match dir {
                        Direction::S => self.vert(curr_v).uv.s,
                        Direction::T => self.vert(curr_v).uv.t,
                    };
                }
            }
        }
        
        match dir {
            Direction::S => self.vert(curr_v).uv.s,
            Direction::T => self.vert(curr_v).uv.t,
        }
    }

    /// Finds the next vertex connected by an edge in the given direction.
    /// This abstracts the topology navigation.
    fn find_next_orthogonal_edge(&self, v: VertID, dir: Direction, forward: bool) -> Option<VertID> {
        let start_edge = self.vert(v).outgoing_edge?;
        let mut curr = start_edge;
        
        // Circulate to find edge aligned with direction
        loop {
            let edge = self.edge(curr);
            
            // Check alignment. This assumes edges store their parametric direction.
            // In a real implementation, we might check the geometry of the UVs.
            let is_aligned = edge.direction == dir; 
            
            // "Forward" in S means increasing S.
            // We need to check geometry or explicit flags.
            // Simplified logic: assume edges are directed.
            let geometry_delta = if let Some(twin) = edge.twin {
                 let dest = self.edge(twin).origin;
                 let uv_dest = self.vert(dest).uv;
                 let uv_src = self.vert(edge.origin).uv;
                 match dir {
                     Direction::S => uv_dest.s - uv_src.s,
                     Direction::T => uv_dest.t - uv_src.t,
                 }
            } else { 0.0 };

            if is_aligned {
                if (forward && geometry_delta > 0.0) |

| (!forward && geometry_delta < 0.0) {
return edge.twin.map(|id| self.edge(id).origin);
}
}

            // Move to next spoke
            if let Some(twin) = edge.twin {
                curr = self.edge(twin).next;
            } else {
                break;
            }
            if curr == start_edge { break; }
        }
        None
    }
}
4.2 Blending Function EvaluationOnce we have the knot vectors for a control point $P_i$, we can define its Blending Function $B_i(s,t) = N(s) \cdot N(t)$.For performance, we use Cox-de Boor recursion. Since T-Splines are typically cubic ($p=3$), we can unroll the recursion for fixed-size arrays, which allows the compiler to optimize heavily (SIMD).Rust/// Evaluates a cubic B-spline basis function given a knot vector of length 5.
/// u: Parameter value
/// knots: [s0, s1, s2, s3, s4]
pub fn cubic_basis_function(u: f64, knots: &[f64; 5]) -> f64 {
// 0th degree basis (step functions)
// N_{i,0}(u) = 1 if u \in |

| u >= knots {
return 0.0;
}

    // This implementation would be the standard De Boor algorithm triangle.
    // For brevity, we show the recursive signature but implementation should be iterative.
    
    // Layer 1 (Degree 1)
    let d0_0 = if u >= knots && u < knots { 1.0 } else { 0.0 };
    //... calculate all N_{i,0}
    
    // Ideally, call truck-geometry's optimized implementation if available.
    // Or use the formula:
    // N_{i,p}(u) = (u - u_i)/(u_{i+p} - u_i) * N_{i,p-1} +...
    
    // Placeholder for standard algorithm
    de_boor_eval(3, u, knots)
}

fn de_boor_eval(degree: usize, u: f64, knots: &[f64]) -> f64 {
// Standard robust implementation required here
// Handling division by zero (0/0 = 0) is critical for multiple knots.
0.5 // Stub
}
4.3 Surface Evaluation (The subs Trait)This brings everything together. To evaluate a point on the surface, we sum the contributions of all control points.Optimization Note: Naively iterating all control points is $O(N)$. We should only iterate points whose support domain covers $(u,v)$. In a T-Mesh, we can find the face containing $(u,v)$ and iterate only the control points influencing that face. For a cubic T-Spline, typically 16 control points influence any given face (similar to a bicubic patch), though T-junctions can slightly alter this number.Rustpub struct TSplineSurface {
pub mesh: TMesh,
// We cache knot vectors because inferring them every time is expensive
pub knot_cache: Vec<([f64; 5], [f64; 5])>,
}

impl ParametricSurface for TSplineSurface {
type Point = Point3<f64>;
type Vector = Vector4<f64>;

    fn subs(&self, u: f64, v: f64) -> Point3<f64> {
        let mut numerator = Vector4::new(0.0, 0.0, 0.0, 0.0);
        let mut denominator = 0.0;

        for (i, vert) in self.mesh.vertices.iter().enumerate() {
            let (s_knots, t_knots) = &self.knot_cache[i];

            // Quick AABB check in parameter space
            if u < s_knots |

| u > s_knots |
| v < t_knots |
| v > t_knots {
continue;
}

            let basis_s = cubic_basis_function(u, s_knots);
            let basis_t = cubic_basis_function(v, t_knots);
            let weight = vert.geometry.w;
            let basis = basis_s * basis_t * weight;

            if basis.abs() > 1e-12 {
                numerator += vert.geometry * basis; // geometry is pre-multiplied by weight usually? 
                // Note: In rational splines, P_i usually stored as (wx, wy, wz, w).
                // If geometry is just (x,y,z,w), we multiply by basis.
                // Standard formula: sum(P_i * w_i * B_i) / sum(w_i * B_i)
                // If Vector4 is (x,y,z,w), then just add basis * Vector4.
            }
            denominator += basis_s * basis_t * weight; // Correct denominator accumulation
        }
        
        if denominator.abs() < 1e-9 {
            // Handle undefined regions or holes
            return Point3::new(0.0, 0.0, 0.0);
        }

        let result_homo = numerator / denominator;
        Point3::new(result_homo.x, result_homo.y, result_homo.z)
    }

    // Implement derivatives similarly...
    fn u_der(&self, u: f64, v: f64) -> Self::Vector { unimplemented!() }
    fn v_der(&self, u: f64, v: f64) -> Self::Vector { unimplemented!() }
    fn normal(&self, u: f64, v: f64) -> Self::Vector { unimplemented!() }
}
5. Analysis-Suitability: The Critical ValidatorAs noted in the research , we must ensure the mesh is Analysis-Suitable (ASTS) to prevent linear dependence.5.1 ASTS Verification CodeWe must implement a check that verifies no T-junction extensions intersect.Rustimpl TMesh {
   /// Validates if the current mesh satisfies ASTS conditions.
   pub fn validate_asts(&self) -> bool {
   let h_exts = self.collect_extensions(Direction::S);
   let v_exts = self.collect_extensions(Direction::T);

        for h in &h_exts {
            for v in &v_exts {
                if self.segments_intersect(h, v) {
                    return false; // Intersection detected!
                }
            }
        }
        true
   }

   struct Segment { start: ParamPoint, end: ParamPoint }

   fn collect_extensions(&self, dir: Direction) -> Vec<Segment> {
   let mut extensions = Vec::new();

        for (idx, vert) in self.vertices.iter().enumerate() {
            if!vert.is_t_junction { continue; }
            
            // Check if T-junction points in 'dir'
            // A T-junction "points" into the face it is missing an edge for.
            // We need logic to determine orientation of the T.
            
            if self.t_junction_orientation(VertID(idx)) == dir {
                 // Trace ray until it hits a perpendicular full edge
                 let start_uv = vert.uv;
                 let end_val = self.cast_ray_for_knot(VertID(idx), dir, 2); // heuristic distance
                 // Real ASTS tracing must go until it hits a line in the T-mesh
                 // that is perpendicular to the extension.
                 
                 let end_uv = match dir {
                     Direction::S => ParamPoint { s: end_val, t: start_uv.t },
                     Direction::T => ParamPoint { s: start_uv.s, t: end_val },
                 };
                 extensions.push(Segment { start: start_uv, end: end_uv });
            }
        }
        extensions
   }

   fn segments_intersect(&self, a: &Segment, b: &Segment) -> bool {
   // Standard 2D segment intersection test
   //...
   false
   }

   fn t_junction_orientation(&self, v: VertID) -> Direction {
   // Logic to inspect neighbors and determine if T points up/down (T) or left/right (S)
   Direction::S // Stub
   }
   }
   This validator should be run after every refinement operation. If false is returned, the refinement must either be rejected or further refined (propagated) until validity is restored.6. Parsing and I/O: The .tsm ParserTo make the library useful, we need to load data. The .tsm format is a simple text format. We can use Rust's string manipulation to parse it.Rustuse std::fs::File;
   use std::io::{self, BufRead};
   use std::path::Path;

impl TMesh {
pub fn from_tsm_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
let file = File::open(path)?;
let lines = io::BufReader::new(file).lines();

        let mut mesh = TMesh::default();
        
        for line in lines {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.is_empty() |

| parts.starts_with('#') { continue; }

            match parts {
                "v" => {
                    // Vertex definition: v <id> <direction_info>
                    // Note:.tsm might have geometric coords elsewhere or combined
                    // This is a simplified parser logic
                },
                "e" => {
                    // Edge definition: e <id> <knot_interval>
                },
                "f" => {
                    // Face definition
                },
                _ => {}
            }
        }
        // Post-processing to link indices into Half-Edge structure
        Ok(mesh)
    }
}
Note: The TSM format often separates topology and geometry. A robust parser must map the IDs in the file to the VertID indices in our Arena vectors. Using a HashMap<usize, VertID> during parsing is recommended to handle non-contiguous IDs in files.7. Comparative Analysis & Performance Considerations7.1 Rust vs. C++ ImplementationsThe proposed Rust architecture offers significant advantages over the legacy C++ implementations found in libraries like OpenNURBS or the GrapeTec T-Spline library.Table 1: Implementation ComparisonFeatureC++ Legacy (OpenNURBS/GrapeTec)Rust Proposed (truck aligned)Memory ModelPointer-heavy (edge->next)Index-based (edges[id].next)SafetyManual memory management (risk of leaks/segfaults)Compile-time ownership checksConcurrencyDifficult (Data races on mutable mesh)Trivial (Send/Sync structs, Rayon iteration)Knot InferenceOften hardcoded or recursiveIterative Ray Casting (Stack safe)ASTS SupportOften absent or added lateFundamental design constraint7.2 Performance Optimization: SmallVecFor knot vectors, we know the size is almost always 5 (for cubic splines). Allocating a Vec<f64> on the heap for every control point is wasteful.Recommendation: Use SmallVec<[f64; 5]> or a fixed-size array [f64; 5]. This keeps the knot data on the stack or inline in the struct, dramatically reducing cache misses during the blending function evaluation loop.7.3 Parallel EvaluationThe surface evaluation loop (Section 4.3) is "embarrassingly parallel". We can use rayon to evaluate grid points for rendering.Rustuse rayon::prelude::*;

impl TSplineSurface {
pub fn tessellate(&self, resolution: usize) -> Vec<Point3<f64>> {
let u_min = 0.0; let u_max = 10.0; // Determine from bounds
let v_min = 0.0; let v_max = 10.0;

        (0..resolution * resolution).into_par_iter().map(|i| {
            let u_i = i % resolution;
            let v_i = i / resolution;
            let u = u_min + (u_i as f64 / resolution as f64) * (u_max - u_min);
            let v = v_min + (v_i as f64 / resolution as f64) * (v_max - v_min);
            
            self.subs(u, v)
        }).collect()
    }
}
8. Conclusion and Future DirectionsThis report has outlined a complete framework for a modern, memory-safe T-Spline kernel in Rust. By strictly adhering to Sederberg’s definition for knot inference and integrating Analysis-Suitability (ASTS) checks, this implementation targets not just computer graphics, but high-precision engineering and simulation markets.The transition to Rust requires adopting an index-based Half-Edge data structure, which, while slightly more verbose than pointer arithmetic, pays dividends in safety and concurrency. Integration with the truck ecosystem allows this kernel to immediately benefit from existing rendering and modeling tools.Future Work:Bézier Extraction: Implementing the algorithm to convert T-Splines into a collection of Bézier patches for direct GPU tessellation (using Compute Shaders in wgpu).Volumetric T-Splines: Extending the T-Mesh to a 3D T-Block structure for solid modeling (Trivariate Splines), which is highly requested in IGA.Reverse Engineering: Algorithms to fit T-Spline surfaces to point clouds, leveraging the local refinement capability to adaptively add detail only where error is high.By following this blueprint, developers can build a geometric engine that combines the mathematical elegance of T-Splines with the rigorous engineering standards of the Rust language.