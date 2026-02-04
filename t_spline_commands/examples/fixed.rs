/*
 * Copyright (C) 2026 Dominick Schroer
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use fixed::types::I10F22;
use num_traits::One;
use std::error::Error;
use t_spline::tmesh::TMesh;
use t_spline::{Command, TSpline};
use t_spline_commands::tessellate::Tessellate;
use t_spline_io::obj_writer::ObjWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let mut spline: TSpline<I10F22> = TSpline::new_unit_square();
    spline.apply_mut(&mut |m: &mut TMesh<I10F22>| m.vertices[0].geometry.z = I10F22::one());

    let points = Tessellate { resolution: 10 }.apply(&spline);

    ObjWriter::default()
        .with_points("Surface", &points)?
        .with_control_surface("Control", &spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
