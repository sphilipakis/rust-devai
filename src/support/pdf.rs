use crate::Result;
use crate::error::Error;
use derive_more::{Deref, From, Into};
use lopdf::{Document, Object, ObjectId, dictionary};
use simple_fs::SPath;
use std::collections::BTreeMap;

#[derive(From, Into, Deref)]
pub struct PdfDoc {
	doc: Document,
}

pub fn load_pdf_doc(path: &SPath) -> Result<PdfDoc> {
	let doc =
		Document::load(path.as_std_path()).map_err(|err| Error::cc(format!("Cannot load pdf doc: {path}"), err))?;

	Ok(PdfDoc { doc })
}

pub fn page_count(pdf: &PdfDoc) -> usize {
	pdf.doc.get_pages().len()
}

/// - `page_num` starts at 1
pub fn extract_pdf_page(pdf: &PdfDoc, page_num: usize) -> Result<PdfDoc> {
	let pages = pdf.get_pages();
	let page_id = pages
		.get(&(page_num as u32))
		.ok_or_else(|| format!("No page found for {page_num}"))?;
	let page_id = *page_id;
	let pdf_page_doc = extract_page(pdf, page_id)?;

	Ok(pdf_page_doc.into())
}

// region:    --- Support

fn extract_page(source_doc: &Document, page_id: ObjectId) -> Result<Document> {
	let mut new_doc = Document::with_version("1.5");

	// -- Map from old ObjectId to new ObjectId
	let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();

	// -- Recursively copy objects that the page depends on
	copy_object_recursive(source_doc, &mut new_doc, page_id, &mut id_map)?;

	// -- Get the new page object id
	let new_page_id = *id_map.get(&page_id).ok_or("Page not found in id_map")?;

	// -- Create Pages object
	let pages_id = new_doc.add_object(dictionary! {
		"Type" => "Pages",
		"Count" => 1,
		"Kids" => vec![Object::Reference(new_page_id)],
	});

	// -- Update page's Parent reference
	if let Ok(page_obj) = new_doc.get_object_mut(new_page_id)
		&& let Object::Dictionary(dict) = page_obj
	{
		dict.set("Parent", Object::Reference(pages_id));
	}

	// -- Create Catalog
	let catalog_id = new_doc.add_object(dictionary! {
		"Type" => "Catalog",
		"Pages" => Object::Reference(pages_id),
	});

	// -- Set trailer
	new_doc.trailer.set("Root", Object::Reference(catalog_id));

	Ok(new_doc)
}

fn copy_object_recursive(
	source_doc: &Document,
	new_doc: &mut Document,
	obj_id: ObjectId,
	id_map: &mut BTreeMap<ObjectId, ObjectId>,
) -> Result<ObjectId> {
	// -- Check if already copied
	if let Some(&new_id) = id_map.get(&obj_id) {
		return Ok(new_id);
	}

	// -- Get source object
	let source_obj = source_doc.get_object(obj_id).map_err(Error::custom)?;

	// -- Reserve a new id first to handle circular references
	let new_id = new_doc.add_object(Object::Null);
	id_map.insert(obj_id, new_id);

	// -- Deep copy the object, updating references
	let copied_obj = copy_object_deep(source_doc, new_doc, source_obj, id_map)?;

	// -- Replace the placeholder with the actual object
	if let Ok(obj) = new_doc.get_object_mut(new_id) {
		*obj = copied_obj;
	}

	Ok(new_id)
}

fn copy_object_deep(
	source_doc: &Document,
	new_doc: &mut Document,
	obj: &Object,
	id_map: &mut BTreeMap<ObjectId, ObjectId>,
) -> Result<Object> {
	match obj {
		Object::Reference(ref_id) => {
			// -- Skip Parent references to avoid circular issues with page tree
			let new_ref_id = copy_object_recursive(source_doc, new_doc, *ref_id, id_map)?;
			Ok(Object::Reference(new_ref_id))
		}

		Object::Array(arr) => {
			let mut new_arr = Vec::with_capacity(arr.len());
			for item in arr {
				new_arr.push(copy_object_deep(source_doc, new_doc, item, id_map)?);
			}
			Ok(Object::Array(new_arr))
		}

		Object::Dictionary(dict) => {
			let mut new_dict = lopdf::Dictionary::new();
			for (key, value) in dict.iter() {
				// -- Skip Parent key to avoid circular references with page tree
				if key == b"Parent" {
					continue;
				}
				let new_value = copy_object_deep(source_doc, new_doc, value, id_map)?;
				new_dict.set(key.clone(), new_value);
			}
			Ok(Object::Dictionary(new_dict))
		}

		Object::Stream(stream) => {
			let mut new_dict = lopdf::Dictionary::new();
			for (key, value) in stream.dict.iter() {
				if key == b"Parent" {
					continue;
				}
				let new_value = copy_object_deep(source_doc, new_doc, value, id_map)?;
				new_dict.set(key.clone(), new_value);
			}
			let new_stream = lopdf::Stream::new(new_dict, stream.content.clone());
			Ok(Object::Stream(new_stream))
		}

		// -- Primitive types can be cloned directly
		Object::Null => Ok(Object::Null),
		Object::Boolean(b) => Ok(Object::Boolean(*b)),
		Object::Integer(i) => Ok(Object::Integer(*i)),
		Object::Real(r) => Ok(Object::Real(*r)),
		Object::Name(n) => Ok(Object::Name(n.clone())),
		Object::String(s, format) => Ok(Object::String(s.clone(), *format)),
	}
}

// endregion: --- Support
