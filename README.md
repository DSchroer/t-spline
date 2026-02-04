# t-spline

A modern t-spline implementation for use in computer graphics and CAD applications.

**Note:** I am releasing this early, there are still many bugs. Use at your own risk. Contributions are welcome.

## Crates

| Crate             | Description                                                          | features                       |
|-------------------|----------------------------------------------------------------------|--------------------------------|
| t-spline          | The data structure and base algorithms of t-splines.                 | **fixed**: Fixed point support |
| t-spline-commands | Commands to modify and interact with t-splines.                      |                                |
| t-spline-io       | Tools for reading and writing 3D data for interop and visualisation. |                                |

## Example

```rs
// create a basic single t-junction spline
let spline = TSpline::new_simple(); 

// Calculate points along the surface of the spline
let points = Tessellate { resolution: 100 }.apply(&spline);

// Print points as obj file
ObjWriter::default()
    .with_points("Points", &points)?
    .write(&mut std::io::stdout())?;
```