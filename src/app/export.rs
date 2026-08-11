//! Export CSV des candidatures filtrées.

pub(super) async fn export_candidatures(
    rows: Vec<crate::modules::candidatures::model::Candidature>,
) -> Result<std::path::PathBuf, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("Exporter les candidatures")
        .set_file_name("candidatures.csv")
        .add_filter("CSV", &["csv"])
        .save_file()
        .await
    else {
        return Err("Export annulé.".into());
    };
    let path = file.path().to_path_buf();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(Vec::new());
    writer
        .write_record([
            "poste",
            "entreprise",
            "contrat",
            "statut",
            "date_envoi",
            "lien_offre",
            "notes",
        ])
        .map_err(|error| format!("Impossible de préparer le CSV : {error}"))?;
    for row in rows {
        writer
            .write_record([
                row.poste,
                row.entreprise_nom.unwrap_or_default(),
                row.type_contrat.to_string(),
                row.statut.to_string(),
                row.date_envoi,
                row.lien_offre.unwrap_or_default(),
                row.notes.unwrap_or_default(),
            ])
            .map_err(|error| format!("Impossible d'écrire le CSV : {error}"))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("Impossible de terminer le CSV : {error}"))?;
    std::fs::write(&path, bytes)
        .map_err(|error| format!("Impossible d'enregistrer le CSV : {error}"))?;
    Ok(path)
}
