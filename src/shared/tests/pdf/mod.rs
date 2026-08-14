//! Helpers communs et déclaration des cas de test.
use super::*;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream, StringFormat};

/// Fabrique en mémoire un PDF minimal d'une page contenant `text`.
fn pdf_avec_texte(text: &str) -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new(
                "Tf",
                vec![Object::Name(b"F1".to_vec()), Object::Integer(12)],
            ),
            Operation::new("Td", vec![Object::Integer(72), Object::Integer(720)]),
            Operation::new(
                "Tj",
                vec![Object::String(
                    text.as_bytes().to_vec(),
                    StringFormat::Literal,
                )],
            ),
            Operation::new("ET", vec![]),
        ],
    };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages_id,
        "Contents" => content_id, "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut buf = Vec::new();
    doc.save_to(&mut buf).unwrap();
    buf
}

/// Fabrique un PDF minimal dont la police CID déclare ses largeurs **au format range**
/// (`c_first c_last w`), comme les PDF produits par LibreOffice.
///
/// Reproduit le bogue amont de `pdf-extract` 0.12.0 : le parsing du tableau `/W` lisait
/// `c_last` et `c_width` depuis `w[i]` au lieu de `w[i + 1]` et `w[i + 2]` ; les largeurs
/// au format range n'étaient jamais insérées, chaque glyphe retombait sur la largeur par
/// défaut `/DW`, et l'heuristique d'espace (écart > 10 % de la taille de police) fabriquait
/// un espace parasite entre une majuscule large (D, C, A) et la lettre suivante.
fn pdf_avec_largeurs_range() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let to_unicode = Stream::new(
        dictionary! {},
        b"/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def
/CMapName /Adobe-Identity-UCS def
/CMapType 2 def
1 begincodespacerange
<0000> <FFFF>
endcodespacerange
1 beginbfrange
<0024> <002D> <0041>
endbfrange
2 beginbfrange
<0044> <0059> <0061>
<00A9> <00AB> <00E7>
endbfrange
endcmap
CMapName currentdict /CMap defineresource pop
end
end"
        .to_vec(),
    );
    let to_unicode_id = doc.add_object(to_unicode);
    let font_desc = doc.add_object(dictionary! {
        "Type" => "FontDescriptor",
        "FontName" => "CAAAAA+LiberationSans",
        "FontBBox" => vec![0.into(), 0.into(), 1000.into(), 1000.into()],
    });
    let cid_font = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "CIDFontType2",
        "BaseFont" => "CAAAAA+LiberationSans",
        "CIDToGIDMap" => "Identity",
        "FontDescriptor" => font_desc,
        "DW" => 500,
        // Format range : `c_first c_last w`. Le « D » (CID 39) fait 722.16797/1000 em,
        // bien plus large que le fallback /DW 500.
        "W" => vec![
            36.into(), 37.into(), 666.99219.into(),
            38.into(), 39.into(), 722.16797.into(),
        ],
    });
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type0",
        "BaseFont" => "CAAAAA+LiberationSans",
        "Encoding" => "Identity-H",
        "DescendantFonts" => vec![cid_font.into()],
        "ToUnicode" => to_unicode_id,
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F6" => font_id },
    });

    // « Développeur » : D = CID 39, é = 0xAB, puis v e l o p p e u r. Chaque glyphe dans
    // son propre `Tj`, séparé par un `Td` égal à sa largeur réelle — exactement la
    // structure des PDF générés par LibreOffice qui déclenche le bogue.
    let content = Content {
        operations: vec![
            Operation::new(
                "cm",
                vec![
                    Object::Real(0.24),
                    Object::Real(0.0),
                    Object::Real(0.0),
                    Object::Real(-0.24),
                    Object::Real(0.0),
                    Object::Real(841.92),
                ],
            ),
            Operation::new("BT", vec![]),
            Operation::new(
                "Tf",
                vec![Object::Name(b"F6".to_vec()), Object::Real(11.02)],
            ),
            Operation::new(
                "Tm",
                vec![
                    Object::Real(1.0),
                    Object::Real(0.0),
                    Object::Real(0.0),
                    Object::Real(-1.0),
                    Object::Real(20.296875),
                    Object::Real(169.0),
                ],
            ),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x27], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(7.955_139), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0xAB], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(6.1263733), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x59], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(5.5078125), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x48], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(6.1263733), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x4F], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(5.5078125), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x52], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(6.1263733), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x53], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(5.5078125), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x53], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(6.1263733), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x48], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(5.5078125), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x58], StringFormat::Hexadecimal)],
            ),
            Operation::new("Td", vec![Object::Real(6.1263733), Object::Integer(0)]),
            Operation::new(
                "Tj",
                vec![Object::String(vec![0x00, 0x55], StringFormat::Hexadecimal)],
            ),
            Operation::new("ET", vec![]),
        ],
    };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages_id,
        "Contents" => content_id, "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut buf = Vec::new();
    doc.save_to(&mut buf).unwrap();
    buf
}

mod test_clean_cv_text_compacte_espaces_et_lignes_vides;
mod test_clean_cv_text_ne_supprime_aucun_contenu;
mod test_extract_text_bytes_invalides_retourne_validation;
mod test_extract_text_pdf_sans_texte_retourne_validation;
mod test_extract_text_pdf_valide_retourne_le_texte;
mod test_les_largeurs_range_ne_fabriquent_pas_d_espace_parasite;
