use cem::{v2, V2, Scene};
use cgmath::{Point3, Matrix4, Deg, InnerSpace};
use std::fmt::{self, Write};
use std::str::FromStr;

// TODO: Date and Time modified
pub const HEADER: &'static str = r#"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <asset>
    <contributor>
      <author>cemconv user</author>
      <authoring_tool>cemconv 0.2.0 collada exporter</authoring_tool>
    </contributor>
    <created>2018-01-01T00:00:00</created>
    <modified>2018-01-01T00:00:00</modified>
    <unit name="meter" meter="1"/>
    <up_axis>Y_UP</up_axis>
  </asset>
  <library_cameras/>
  <library_images/>
  <library_geometries>
"#;

const FORMAT_POS: &'static str = r##"<param name="X" type="float"/><param name="Y" type="float"/><param name="Z" type="float"/>"##;
const FORMAT_TEX: &'static str = r##"<param name="S" type="float"/><param name="T" type="float"/>"##;

struct Geometry<'n> {
	// Name
	name: &'n str,
	// Position (X, Y, Z)
	mesh_positions: Vec<f32>,
	// Normal (X, Y, Z)
	mesh_normals: Vec<f32>,
	// Texture (S, T)
	mesh_map: Vec<f32>,
	// Indices (V1, V2, V3)
	polygons: Vec<u32>
}

impl<'n> fmt::Display for Geometry<'n> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, r#"    <geometry id="{0}-mesh" name="{0}">"#, self.name)?;
		writeln!(f, r#"      <mesh>"#)?;

		let vertex_count = self.mesh_positions.len() / 3;

		{
			let mut write_source = |source: &str, array: &[f32], stride: usize, format: &str| -> fmt::Result {
				writeln!(f, r#"        <source id="{}-{}">"#, self.name, source)?;

				writeln!(f, r#"          <float_array id="{}-{}-array" count="{}">"#, self.name, source, array.len())?;
				for position in array {
					write!(f, "{:.8} ", position)?;
				}
				writeln!(f, r#"          </float_array>"#)?;
				writeln!(f, r##"<technique_common><accessor source="#{}-{}-array" count="{}" stride="{}">"##, self.name, source, vertex_count, stride)?;
				writeln!(f, "{}", format)?;
				writeln!(f, r##"</accessor></technique_common>"##)?;
				writeln!(f, r#"        </source>"#)
			};

			write_source("mesh-positions", &self.mesh_positions, 3, FORMAT_POS)?;
			write_source("mesh-normals", &self.mesh_normals, 3, FORMAT_POS)?;
			write_source("mesh-map", &self.mesh_map, 2, FORMAT_TEX)?;
		}

		writeln!(f, r##"        <vertices id="{0}-mesh-vertices"><input semantic="POSITION" source="#{0}-mesh-positions"/></vertices>"##, self.name)?;

		writeln!(f, r#"        <triangles count="{}">"#, self.polygons.len() / 3)?;
		writeln!(f, r##"          <input semantic="VERTEX" source="#{}-mesh-vertices" offset="0"/>"##, self.name)?;
		writeln!(f, r##"          <input semantic="NORMAL" source="#{}-mesh-normals" offset="1"/>"##, self.name)?;
		writeln!(f, r##"          <input semantic="TEXCOORD" source="#{}-mesh-map" offset="2" set="0"/>"##, self.name)?;

		write!(f, r#"          <p>"#)?;
		for index in &self.polygons {
			write!(f, "{0} {0} {0} ", index)?;
		}
		writeln!(f, r#"          </p>"#)?;
		writeln!(f, r#"        </triangles>"#)?;
		writeln!(f, r#"      </mesh>"#)?;
		write!(f, r#"    </geometry>"#)?;


		Ok(())
	}
}

struct Light {
	color: (f32, f32, f32),
	unk: (u32, u32, u32)
}

impl FromStr for Light {
	type Err = &'static str;

	fn from_str(definition: &str) -> Result<Self, Self::Err> {
		fn split(definition: &str) -> Option<(&str, &str, &str, &str, &str, &str)> {
			let mut split = definition.split('_');

			split.next()?; // "light"

			let r = split.next()?;
			let g = split.next()?;
			let b = split.next()?;

			let i = split.next()?;
			let j = split.next()?;
			let k = split.next()?;

			Some((r, g, b, i, j, k))
		};

		let (r, g, b, i, j, k) = split(definition).ok_or("Invalid light definition")?;

		let (r, g, b, i, j, k) = (
			r.parse::<u32>().map_err(|_| "failed to parse number")?,
			g.parse::<u32>().map_err(|_| "failed to parse number")?,
			b.parse::<u32>().map_err(|_| "failed to parse number")?,
			i.parse::<u32>().map_err(|_| "failed to parse number")?,
			j.parse::<u32>().map_err(|_| "failed to parse number")?,
			k.parse::<u32>().map_err(|_| "failed to parse number")?
		);

		Ok(Light {
			color: (
				(r as f32) / 255.0,
				(g as f32) / 255.0,
				(b as f32) / 255.0
			),
			unk: (i, j, k)
		})
	}
}


fn write_meshes(name: &str, model: &V2, string: &mut String) {
	let triangle_data = &model.lod_levels[0];
	let mut polygons = vec![0; model.lod_levels[0].len() * 3];

	for &v2::Material { name: ref _name, texture: _texture, ref triangles, vertex_offset, vertex_count: _vertex_count, texture_name: ref _texture_name } in &model.materials {
		let triangle_slice = triangles[0];

		for index in 0..triangle_slice.len {
			let index = index + triangle_slice.offset;
			let triangle = &triangle_data[index as usize];

			let indices = (
				vertex_offset + triangle.0,
				vertex_offset + triangle.1,
				vertex_offset + triangle.2
			);

			polygons[(index as usize)*3 + 0] = indices.0;
			polygons[(index as usize)*3 + 1] = indices.1;
			polygons[(index as usize)*3 + 2] = indices.2;
		}
	}

	for (frame_index, frame) in model.frames.iter().enumerate() {
		let framed_name = format!("{}_frame{}", name, frame_index);

		let mut geometry = Geometry {
			name: if frame_index > 0 { &framed_name } else { name },
			mesh_positions: vec![0.0; frame.vertices.len() * 3],
			mesh_normals: vec![0.0; frame.vertices.len() * 3],
			mesh_map: vec![0.0; frame.vertices.len() * 2],
			polygons: polygons.clone()
		};

		let transform = Matrix4::from_angle_x(Deg(-90.0));

		for (index, vertex) in frame.vertices.iter().enumerate() {
			let normal = (transform * vertex.normal.normalize().extend(0.0)).truncate();
			let position = Point3::from_homogeneous(transform * vertex.position.to_homogeneous());

			geometry.mesh_positions[index*3 + 0] = position.x;
			geometry.mesh_positions[index*3 + 1] = position.y;
			geometry.mesh_positions[index*3 + 2] = position.z;

			geometry.mesh_normals[index*3 + 0] = normal.x;
			geometry.mesh_normals[index*3 + 1] = normal.y;
			geometry.mesh_normals[index*3 + 2] = normal.z;

			geometry.mesh_map[index*2 + 0] = vertex.texture.x;
			geometry.mesh_map[index*2 + 1] = 1.0 - vertex.texture.y;
		}

		writeln!(string, "{}", geometry).unwrap();
	}
}

pub fn convert(cem: Scene<V2>) -> String {
	let mut string = String::new();

	string.push_str(HEADER);

	write_meshes("Scene_Root", &cem.model, &mut string);

	string.push_str("  </library_geometries>\n");

	string.push_str("  <library_lights>\n");

	for name in &cem.model.tag_points {

		writeln!(string, "    <light id=\"{}-light\"><technique_common>\n", name).unwrap();

		if name.starts_with("light_") {
			match name.parse::<Light>() {
				Ok(light) => {

					writeln!(string, "    <point><color>{} {} {}</color><linear_attenuation>0.3</linear_attenuation></point>\n", light.color.0, light.color.1, light.color.2).unwrap();
					string.push_str("    </technique_common></light>\n");

					continue;
				}
				Err(message) => eprintln!("Failed to parse light \"{}\": {}", name, message)
			}
		}

		string.push_str("    <point><color>1.0 1.0 1.0</color><linear_attenuation>0.3</linear_attenuation></point>\n");
		string.push_str("    </technique_common></light>\n");
	}

	string.push_str("  </library_lights>\n");

	string.push_str("  <library_controllers>\n");

	let name = "Scene_Root"; // TODO
	let model = &cem.model;

	if cem.model.frames.len() > 1 {
		writeln!(string, "    <controller id=\"{0}-morph\" name=\"{0}-morph\">", name).unwrap();
		writeln!(string, "      <morph source=\"#{}-mesh\" method=\"NORMALIZED\">", name).unwrap();

		// Targets Array
		writeln!(string, "        <source id=\"{}-targets\">", name).unwrap();
		writeln!(string, "          <IDREF_array id=\"{}-targets-array\" count=\"{}\">", name, model.frames.len()-1).unwrap();

		for frame_index in 1..model.frames.len() {
			writeln!(string, "            {}_frame{}-mesh", name, frame_index).unwrap();
		}

		string.push_str("          </IDREF_array>\n");
		writeln!(string, r##"<technique_common><accessor source="#{}-targets-array" count="{}" stride="1"><param name="IDREF" type="IDREF"/></accessor></technique_common>"##, name, model.frames.len()-1).unwrap();

		string.push_str("        </source>\n");

		// Weights Array
		writeln!(string, "        <source id=\"{}-weights\">", name).unwrap();
		write!(string, "          <float_array id=\"{}-weights-array\" count=\"{}\">", name, model.frames.len()-1).unwrap();

		for _ in 1..model.frames.len() {
			string.push_str("0 ");
		}

		string.push_str("</float_array>\n");
		writeln!(string, r##"<technique_common><accessor source="#{}-weights-array" count="{}" stride="1"><param name="MORPH_WEIGHT" type="float"/></accessor></technique_common>"##, name, model.frames.len()-1).unwrap();

		string.push_str("        </source>\n");

		string.push_str("        <targets>\n");
		writeln!(string, "          <input semantic=\"MORPH_TARGET\" source=\"#{}-targets\"/>", name).unwrap();
		writeln!(string, "          <input semantic=\"MORPH_WEIGHT\" source=\"#{}-weights\"/>", name).unwrap();
		string.push_str("        </targets>\n");
		string.push_str("      </morph>\n");
		string.push_str("    </controller>\n");
	}

	string.push_str("  </library_controllers>\n");

	string.push_str(r##"  <library_visual_scenes><visual_scene id="Scene" name="Scene">"##);
	string.push('\n');

	writeln!(string, r##"<node id="{0}" name="{0}" type="NODE"><matrix sid="transform">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix><instance_geometry url="#{0}-mesh"/>"##, name).unwrap();

	{
		let transform = Matrix4::from_angle_x(Deg(-90.0));

		for (tag_name, position) in cem.model.tag_points.iter().zip(cem.model.frames[0].tag_points.iter()) {
			let position = Point3::from_homogeneous(transform * position.to_homogeneous());

			writeln!(string, "    <node name=\"{}\">\n", tag_name).unwrap();
			writeln!(string, "    <translate>{} {} {}</translate>", position.x, position.y, position.z).unwrap();
			writeln!(string, "    <instance_light url=\"#{}-light\" />\n", tag_name).unwrap();
			string.push_str("</node>");
		}
	}

	string.push_str("</node>");

	string.push_str(r##"  </visual_scene></library_visual_scenes>"##);
	string.push('\n');

	string.push_str(r##"  <scene><instance_visual_scene url="#Scene"/></scene>"##);
	string.push('\n');
	string.push_str("</COLLADA>");

	string
}