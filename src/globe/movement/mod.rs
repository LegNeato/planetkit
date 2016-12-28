mod triangles;
mod transform;
#[cfg(test)]
mod test;

use na;

use super::{ IntCoord, CellPos, Dir };
use super::cell_shape::NEIGHBOR_OFFSETS;

use self::triangles::*;
use self::transform::*;

// See `triangles.rs` for discussion about the approach used here,
// and the list of all triangles used.

/// Returns an error if `pos` and `dir` do not point at an edge
/// of the current cell; it's illegal to move toward a cell vertex.
///
/// Behaviour is undefined if `pos` and `dir` are not already in their
/// canonical form; i.e. if `pos` is on the boundary of two root quads,
/// then `dir` points into that quad or along its edge, but not out.
///
/// This extends to behaviour being undefined if `pos` lies outside the
/// bounds of its root quad.
///
/// Both these cases will panic in debug mode.
pub fn move_forward(
    pos: &mut CellPos,
    dir: &mut Dir,
    resolution: [IntCoord; 2],
) -> Result<(), ()> {
    debug_assert_pos_within_root(pos, resolution);

    // Only allow moving into immediately adjacent cells.
    //
    // We've already established at this point that we will be moving
    // to a cell that is within the same root quad as the one we are
    // already in. The special nature of the 12 pentagons is only relevant
    // when considering the interface between quads, so for this part we
    // can treat both cells as hexagons.
    if !dir.points_at_hex_edge() {
        return Err(());
    }

    *pos = adjacent_pos_in_dir(*pos, *dir)?;

    // Pos should still be within the root bounds; otherwise it means
    // `pos` and `dir` were not in their canonical forms when passed
    // into this function. (`pos` should have been in a different root.)
    debug_assert_pos_within_root(pos, resolution);

    // Rebase on whatever root quad we're now pointing into if necessary.
    maybe_rebase_on_adjacent_root(pos, dir, resolution);

    // Nothing bad happened up to here; presumably we successfully
    // advanced by one step and rebased on whatever root quad we're
    // now pointing into if necessary.
    Ok(())
}

/// Returns an error if `pos` and `dir` do not point at an edge
/// of the current cell; this is intended for rotating to valid
/// directions for forward movement, which isnt possible if the starting
/// direction is illegal.
///
/// Behaviour is undefined if `pos` and `dir` are not already in their
/// canonical form; i.e. if `pos` is on the boundary of two root quads,
/// then `dir` points into that quad or along its edge, but not out.
///
/// This extends to behaviour being undefined if `pos` lies outside the
/// bounds of its root quad.
///
/// Both these cases will panic in debug mode.
pub fn turn_left_by_one_hex_edge(
    pos: &mut CellPos,
    dir: &mut Dir,
    resolution: [IntCoord; 2],
) -> Result<(), ()> {
    debug_assert_pos_within_root(pos, resolution);

    // Only allow turning from and to valid directions for forward movement.
    //
    // We've already established at this point that we will be moving
    // to a cell that is within the same root quad as the one we are
    // already in. The special nature of the 12 pentagons is only relevant
    // when considering the interface between quads, so for this part we
    // can treat both cells as hexagons.
    if !dir.points_at_hex_edge() {
        return Err(());
    }

    #[cfg(debug)]
    {
        let next_pos = adjacent_pos_in_dir(*pos, *dir)?;
        // Pos should still be within the root bounds; otherwise it means
        // `pos` and `dir` were not in their canonical forms when passed
        // into this function. (`pos` should have been in a different root.)
        debug_assert_pos_within_root(next_pos, resolution);
    }

    // Turn left by one hexagon edge.
    *dir = dir.next_hex_edge_left();

    // Rebase on whatever root quad we're now pointing into if necessary.
    maybe_rebase_on_adjacent_root(pos, dir, resolution);

    // Nothing bad happened up to here; presumably we successfully
    // turned by one hexagon edge and rebased on whatever root quad we're
    // now pointing into if necessary.
    Ok(())
}

/// Returns an error if `pos` and `dir` do not point at an edge
/// of the current cell; this is intended for rotating to valid
/// directions for forward movement, which isnt possible if the starting
/// direction is illegal.
///
/// Behaviour is undefined if `pos` and `dir` are not already in their
/// canonical form; i.e. if `pos` is on the boundary of two root quads,
/// then `dir` points into that quad or along its edge, but not out.
///
/// This extends to behaviour being undefined if `pos` lies outside the
/// bounds of its root quad.
///
/// Both these cases will panic in debug mode.
pub fn turn_right_by_one_hex_edge(
    pos: &mut CellPos,
    dir: &mut Dir,
    resolution: [IntCoord; 2],
) -> Result<(), ()> {
    debug_assert_pos_within_root(pos, resolution);

    // Only allow turning from and to valid directions for forward movement.
    //
    // We've already established at this point that we will be moving
    // to a cell that is within the same root quad as the one we are
    // already in. The special nature of the 12 pentagons is only relevant
    // when considering the interface between quads, so for this part we
    // can treat both cells as hexagons.
    if !dir.points_at_hex_edge() {
        return Err(());
    }

    #[cfg(debug)]
    {
        let next_pos = adjacent_pos_in_dir(*pos, *dir)?;
        // Pos should still be within the root bounds; otherwise it means
        // `pos` and `dir` were not in their canonical forms when passed
        // into this function. (`pos` should have been in a different root.)
        debug_assert_pos_within_root(next_pos, resolution);
    }

    // Turn right by one hexagon edge.
    *dir = dir.next_hex_edge_right();

    // Rebase on whatever root quad we're now pointing into if necessary.
    maybe_rebase_on_adjacent_root(pos, dir, resolution);

    // Nothing bad happened up to here; presumably we successfully
    // turned by one hexagon edge and rebased on whatever root quad we're
    // now pointing into if necessary.
    Ok(())
}

/// Get next cell in direction faced by `dir`, without considering
/// movement between roots. Note that this may therefore return positions
/// outside the boundaries of `pos`'s current root.
///
/// Returns an error if `dir` does not point to a direction that would
/// represent an immediately adjacent cell if in a hexagon. (Movement
/// toward vertices is undefined.)
fn adjacent_pos_in_dir(
    pos: CellPos,
    dir: Dir,
) -> Result<CellPos, ()> {
    if !dir.points_at_hex_edge() {
        return Err(());
    }

    // Find the (x, y) offset for `dir` and apply to `pos`.
    // Edge index is half the direction index, because direction 0
    // points at edge 0.
    let edge_index = (dir.index / 2) as usize;
    let (dx, dy) = NEIGHBOR_OFFSETS[edge_index];
    Ok(CellPos {
        root: pos.root,
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z,
    })
}

/// Assumes `pos` is either exactly on the interface between
/// two root quads, or somewhere within a root.
///
/// Panics if `dir` does not point to a direction that would
/// represent an immediately adjacent cell _if `pos` were if in a hexagon_
/// (which is not necessarily so).
fn maybe_rebase_on_adjacent_root(
    pos: &mut CellPos,
    dir: &mut Dir,
    resolution: [IntCoord; 2],
) {
    // First check if `pos` clearly doesn't need to be re-based on a neighboring
    // root quad; i.e. it's not on the edge.
    //
    // Prefer this to falling through to this case so that we can avoid all the
    // computation below, and also detect movement cases we've missed and panic.
    // (See bottom of function.)
    let away_from_root_edges =
        pos.x > 0 &&
        pos.y > 0 &&
        pos.x < resolution[0] &&
        pos.y < resolution[1];
    if away_from_root_edges {
        return;
    }

    // TODO: moving onto pentagons doesn't really fit well with this at all. :/
    //
    // Consider moving from (0, 0) in the positive y-axis. In the first pentagon
    // you meet, naively ploughing ahead with rebase code that doesn't have
    // any idea where you _came from_ will happily assume we came from triangle 6,
    // and do nothing at all. So the simple approach of "move or rotate,
    // and then separately rebase" is not actually valid.

    let tri = if is_pentagon(pos, resolution) {
        // Pick the triangle whose middle axis is closest to dir.
        // This is mostly useful when we're rotating.
        triangle_on_pos_with_closest_mid_axis(pos, dir, resolution)
    } else {
        // Pick the closest triangle that is oriented such that `pos` lies
        // between its x-axis and y-axis.
        closest_triangle_to_point(pos, resolution)
    };

    // Transform `pos` and `dir` to be relative to triangle apex;
    // i.e. so we can treat them as if they're in arctic space,
    // and then transform them back when we're done.
    let (new_pos, new_dir) = world_to_local(*pos, *dir, resolution, tri);
    *pos = new_pos;
    *dir = new_dir;

    let next_pos = adjacent_pos_in_dir(
        pos.clone(), dir.clone()
    ).expect("Caller should have assured we're pointing at a hex edge.");

    // Next check if `pos` doesn't need to be re-based on a neighboring root quad
    // because it's `next_pos` is still in this root. Note that we're not checking
    // the far edges because re-orienting `pos` and `next_pos` on arctic-equivalent
    // triangles like we're doing guarantees they'll never be near the far end.
    //
    // Prefer this to falling through to this case so that we can avoid all the
    // computation below, and also detect movement cases we've missed and panic.
    // (See bottom of function.)
    let still_in_same_quad =
        next_pos.x >= 0 &&
        next_pos.y >= 0;
    if still_in_same_quad {
        transform_into_exit_triangle(pos, dir, resolution, &tri.exits[0]);
        return;
    }

    // Moving east around north pole.
    if next_pos.x < 0 {
        pos.x = pos.y;
        pos.y = 0;
        *dir = dir.next_hex_edge_right();

        transform_into_exit_triangle(pos, dir, resolution, &tri.exits[1]);
        return;
    }

    // Moving west around north pole.
    if next_pos.y < 0 {
        pos.y = pos.x;
        pos.x = 0;
        *dir = dir.next_hex_edge_left();

        transform_into_exit_triangle(pos, dir, resolution, &tri.exits[4]);
        return;
    }

    // Uh oh, we must have missed a case.
    panic!("Oops, we must have forgotten a movement case. Sounds like we didn't test hard enough!")
}

// Transform (x, y, dir) back to local coordinates near where we started,
// taking account any change in root required.
fn transform_into_exit_triangle(
    pos: &mut CellPos,
    dir: &mut Dir,
    resolution: [IntCoord; 2],
    exit: &Exit,
) {
    let exit_tri = &TRIANGLES[exit.triangle_index];
    pos.root.index = (pos.root.index + exit.root_offset) % 5;
    let (new_pos, new_dir) = local_to_world(*pos, *dir, resolution, exit_tri);
    *pos = new_pos;
    *dir = new_dir;
}

/// Pick triangle with the closest apex that is oriented such that `pos` lies
/// between its x-axis and y-axis.
///
/// If `pos` is on a pentagon, you probably won't want this.
/// Consider `triangle_on_pos_with_closest_mid_axis` instead?
fn closest_triangle_to_point(
    pos: &CellPos,
    resolution: [IntCoord; 2],
) -> &'static Triangle {
    // First we filter down to those where
    // pos lies between the triangle's x-axis and y-axis.
    // (See diagram in `triangles.rs`.)
    //
    // It is important that we don't pick a differently-oriented
    // triangle with the same apex, because that would sometimes
    // lead us to unnecessarily transforming positions into
    // neighboring quads. (We try to maintain stability within a given
    // quad in general, and there's a bunch of logic around here in particular
    // that assumes that.)
    let candidate_triangles = if pos.x + pos.y < resolution[0] {
        &TRIANGLES[0..3]
    } else if pos.y < resolution[0] {
        &TRIANGLES[3..6]
    } else if pos.x + pos.y < resolution[1] {
        &TRIANGLES[6..9]
    } else {
        &TRIANGLES[9..12]
    };

    // Pick the closest triangle.
    type Pos2 = na::Point2<IntCoord>;
    let pos2 = Pos2::new(pos.x, pos.y);
    candidate_triangles.iter().min_by_key(|triangle| {
        // Both parts of the apex are expressed in terms of x-dimension.
        let apex = triangle.apex * resolution[0];
        let apex_to_pos = na::Absolute::abs(&(pos2 - apex));
        let hex_distance_from_apex_to_pos = apex_to_pos.x + apex_to_pos.y;
        hex_distance_from_apex_to_pos
    }).expect("There should have been exactly three items; this shouldn't be possible!")
}

/// For whatever 1-3 triangles `pos` is sitting atop, find the one
/// whose "middle axis" (half-way between x-axis and y-axis) is closest
/// to `dir`.
///
/// This is useful for re-basing while turning, without unnecessarily
/// re-basing into a neighbouring root.
///
/// Panics called with any pos that is not a pentagon.
fn triangle_on_pos_with_closest_mid_axis(
    pos: &CellPos,
    dir: &Dir,
    resolution: [IntCoord; 2],
) -> &'static Triangle {
    // If `pos` sits on a pentagon and we're re-basing, then that probably
    // means we're turning. Because we're on a pentagon, it's important that
    // we select the triangle that is most closely oriented to our direction,
    // so that we don't accidentally re-base into a neighbouring quad unnecessarily.
    // (We try to maintain stability within a given quad in general, and there's a
    // bunch of logic around here in particular that assumes that.)
    type Pos2 = na::Point2<IntCoord>;
    let pos2 = Pos2::new(pos.x, pos.y);
    TRIANGLES.iter()
        .filter(|triangle| {
            // There will be between one and three triangles that
            // we are exactly on top of.
            use num_traits::Zero;
            // Both parts of the apex are expressed in terms of x-dimension.
            let apex = triangle.apex * resolution[0];
            let apex_to_pos = na::Absolute::abs(&(pos2 - apex));
            apex_to_pos.is_zero()
        })
        .min_by_key(|triangle| {
            // Find triangle with minimum angle between its "mid axis"
            // and wherever `pos` is pointing.
            let middle_axis_dir: i16 = (triangle.x_dir as i16 + 1) % 12;
            let mut a = middle_axis_dir - dir.index as i16;
            if a > 6 {
                a -= 12;
            } else if a < -6 {
                a += 12;
            }
            a.abs()
        }).expect("There should have been 1-3 triangles; did you call this with a non-pentagon pos?")
}

pub fn is_pentagon(pos: &CellPos, resolution: [IntCoord; 2]) -> bool {
    pos.x == 0 && pos.y == 0 ||
    pos.x == 0 && pos.y == resolution[0] ||
    pos.x == 0 && pos.y == resolution[1] ||
    pos.x == resolution[0] && pos.y == 0 ||
    pos.x == resolution[0] && pos.y == resolution[0] ||
    pos.x == resolution[0] && pos.y == resolution[1]
}

fn debug_assert_pos_within_root(
    pos: &mut CellPos,
    resolution: [IntCoord; 2],
) {
    debug_assert!(
        pos.x >= 0 &&
        pos.y >= 0 &&
        pos.x <= resolution[0] &&
        pos.y <= resolution[1],
        "`pos` was outside its root at the given resolution."
    );
}

// TODO: `turn_left` and `turn_right` functions that are smart
// about pentagons.
