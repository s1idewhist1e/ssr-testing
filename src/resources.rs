use crate::{model::Material, texture::Texture};

use std::{
    ffi::OsString,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use cgmath::{InnerSpace, Vector2};
use gltf::Gltf;
use wgpu::util::DeviceExt;

use crate::{model, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    let txt = {
        let url = format_url(file_name);
        reqwest::get(url).await?.text().await?
    };
    #[cfg(not(target_arch = "wasm32"))]
    let txt = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    let data = {
        let url = format_url(file_name);
        reqwest::get(url).await?.bytes().await?.to_vec()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<Texture> {
    let fallback_texture = image::DynamicImage::new(1, 1, image::ColorType::Rgba8);

    log::debug!("Loading texture from file: {:?}", file_name);

    let path = Path::new(file_name);

    let extension = path.extension();

    let data = load_binary(file_name).await;

    match data {
        Ok(data) => {
            if let Some(ext) = extension
                && ext == "tga"
            {
                let image = image::load_from_memory_with_format(&data, image::ImageFormat::Tga)?;
                return Texture::from_image(device, queue, &image, Some(file_name));
            }
            Texture::from_bytes(device, queue, &data, file_name)
        }
        Err(_) => {
            log::info!(
                "Unable to open texture '{}', using fallback texture instead",
                file_name
            );
            return Texture::from_image(device, queue, &fallback_texture, Some(file_name));
        }
    }
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);

    let mut obj_reader = BufReader::new(obj_cursor);
    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
        |p| async move {
            let p_buf = PathBuf::from(file_name);
            let dirname = p_buf.parent().unwrap();

            let mut path = PathBuf::from(dirname);
            path.push(p);

            log::debug!("Loading materials file: {:?}", path);
            let mat_text = load_string(&path.as_os_str().to_str().unwrap())
                .await
                .unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let p_buf = PathBuf::from(file_name);
        let dirname = p_buf.parent().unwrap();

        let mut path = PathBuf::from(dirname);
        path.push(m.diffuse_texture);

        let diffuse_texture =
            load_texture(path.as_os_str().to_str().unwrap(), device, queue).await?;
        let normal_texture = load_texture(
            &m.unknown_param.get("map_Disp").unwrap_or(&String::new()),
            device,
            queue,
        )
        .await?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            normal_texture,
            bind_group,
        });
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0., 0., 0.],
                            tangent: [0.; 3],
                            bitangent: [0.; 3],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [m.mesh.texcoords[i * 2], 1. - m.mesh.texcoords[i * 2 + 1]],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                            tangent: [0.; 3],
                            bitangent: [0.; 3],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Generate tangent and bitangent
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let vec1 = pos1 - pos0;
                let vec2 = pos2 - pos0;
                // let vec2 = vec1.cross(cgmath::Vector3::from(v0.normal));
                //let vec2 = pos2 - pos0;

                let vec_uv1 = uv1 - uv0;
                // let vec_uv2 = Vector2::from((-vec_uv1.y, vec_uv1.x));
                let vec_uv2 = uv2 - uv0;

                let r = 1.0 / (vec_uv1.x * vec_uv2.y - vec_uv1.y * vec_uv2.x);
                let tangent = (vec1 * vec_uv2.y - vec2 * vec_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (vec2 * vec_uv1.x - vec1 * vec_uv2.x) * -r;

                vertices[c[0] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                let tangent = cgmath::Vector3::from(v.tangent) * denom;
                let bitangent = cgmath::Vector3::from(v.bitangent) * denom;
                let normal = cgmath::Vector3::from(v.normal);

                println!(
                    "Normal error: \n\t expected: {:?} \n\t got: {:?} \n\t dot product: {:?}",
                    normal.normalize(),
                    tangent.cross(bitangent).normalize(),
                    normal.normalize().dot(tangent.cross(bitangent).normalize())
                );
                v.tangent = tangent.into();
                v.bitangent = bitangent.into();
                // v.normal = tangent.cross(bitangent).normalize().into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: &bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

// pub async fn load_model_gltf(
//     file_name: &str,
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     layout: &wgpu::BindGroupLayout,
// ) -> anyhow::Result<model::Model> {
//     let (document, buffers, images) = gltf::import(file_name)?;
//
//     let materials = document
//         .materials()
//         .enumerate()
//         .map(|(i, mat)|{
//
//             let texture_path =
//          model::Material {
//             name: mat
//                 .name()
//                 .unwrap_or(&format!("Material {} of {}", i, file_name))
//                 .into(),
//         }});
//     let meshes = document.meshes().enumerate().map(|(i, mesh)| {
//         let primitives = mesh.primitives();
//
//         for primitive in primitives {
//             let material = primitive.material();
//             let attributes = primitive.attributes().collect::<Vec<_>>();
//         }
//
//         model::Mesh {
//             name: mesh
//                 .name()
//                 .unwrap_or(&format!("Mesh {} of file {}", i, file_name))
//                 .into(),
//         }
//     });
//
//     todo!()
// }
