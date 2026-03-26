fn parse_stl_binary(body: &[u8], tri_count: usize) -> Result<Vec<Triangle>, MeshError> {
    let (_, triangles) = count(parse_triangle, tri_count)(body).map_err(|e| {
        MeshError::ParseError {
            line: 0,
            message: format!("failed to parse binary STL triangle data: {}", e),
        }
    })?;
    Ok(triangles)
}

// Previous code logic should be invoked in the appropriate place where parse_stl_binary is called.