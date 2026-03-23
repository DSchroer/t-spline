Based on the provided paper, rendering a point $(s,t)$ on a T-spline involves treating it as a PB-spline (Point-Based spline) where the knot vectors for each control
  point are locally determined by the topology of the T-mesh.

  To calculate the surface position $\mathbf{P}(s,t)$, follow these steps:

  1. Determine Local Knot Vectors
  For each control point $\mathbf{P}_i$ in the T-mesh, you must derive two local knot vectors: $\mathbf{s}_i = [s_{i0}, s_{i1}, s_{i2}, s_{i3}, s_{i4}]$ and $\mathbf{t}_i
  = [t_{i0}, t_{i1}, t_{i2}, t_{i3}, t_{i4}]$.
   * Origin: The central knots $(s_{i2}, t_{i2})$ are the parameter coordinates of the control point $\mathbf{P}_i$ itself.
   * Search: To find the remaining knots (e.g., $s_{i3}$ and $s_{i4}$), shoot a ray in parameter space from $(s_{i2}, t_{i2})$ in the corresponding direction (e.g., the
     positive $s$ direction). The knots are the parameter coordinates of the first two $s$-edges (vertical edges) the ray intersects. The same process applies to all
     directions for both $s$ and $t$.

  2. Evaluate Basis Functions
  For each control point $\mathbf{P}_i$, the basis function $B_i(s,t)$ is the product of two univariate cubic B-spline basis functions:
  $$B_i(s,t) = N_{i}^3(s) N_{i}^3(t)$$
   * $N_{i}^3(s)$ is the cubic B-spline basis function defined over the local knot vector $\mathbf{s}_i$.
   * $N_{i}^3(t)$ is the cubic B-spline basis function defined over the local knot vector $\mathbf{t}_i$.

  3. Calculate the Surface Point
  The surface is defined as a rational blend of the control points. Use the following equation to find the coordinate at $(s,t)$:
  $$\mathbf{P}(s,t) = \frac{\sum_{i=1}^{n} \mathbf{P}_i B_i(s,t)}{\sum_{i=1}^{n} B_i(s,t)}$$
   * Standard T-splines: If the T-mesh is "standard" (e.g., it was created through valid local knot insertions or merging B-splines), the basis functions will sum to one
     ($\sum B_i = 1$), and the denominator can be omitted, reducing the surface to a non-rational form.

  Summary for Implementation
  To render the entire surface, you typically tessellate it by:
   1. Identifying the Bézier domains (rectangles formed by extending every T-junction's influence by two "bays" in the grid).
   2. Performing local knot insertion to extract the Bézier control points for each domain.
   3. Rendering the resulting bi-cubic Bézier patches using standard GPU techniques.
