impl crate::app::CopaibaApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_mvt_auto_oto(&mut self) {
        self.mvt_auto_oto.source_oto_path = self.cur().oto_path.clone();
        let selected_indices: std::collections::HashSet<usize> = {
            let tab = self.cur();
            if tab.multi_selection.is_empty() {
                tab.filtered.get(tab.selected).copied().into_iter().collect()
            } else {
                tab.multi_selection.iter().copied().collect()
            }
        };
        let Some(dir) = self.cur().oto_dir.clone() else {
            self.mvt_auto_oto.status = "Abra um oto.ini com os WAVs acessíveis no disco.".into();
            return;
        };
        let japanese = self.config.mvt_auto_oto.language.to_lowercase().starts_with("ja");
        let scheme = self.config.mvt_auto_oto.oto_scheme;
        let selected_files: std::collections::HashSet<String> = {
            let tab = self.cur();
            selected_indices.iter().filter_map(|index| tab.entries.get(*index))
                .map(|entry| entry.filename.clone()).collect()
        };
        let items = {
            let tab = self.cur();
            tab.entries.iter().enumerate().filter_map(|(index, entry)| {
                if !selected_files.contains(&entry.filename) { return None; }
                let stem = std::path::Path::new(&entry.filename).file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let filename_alias = entry.alias.trim() == stem;
                let segmented_japanese_stem = japanese && filename_alias
                    && stem.split('_').filter(|part| !part.is_empty()).count() >= 2
                    && stem.chars().all(|ch| ch == '_' || ch.is_ascii_alphabetic());
                let inferred_vcv = crate::app::mvt_auto_oto::looks_like_vcv_alias(&entry.alias)
                    && !crate::app::mvt_auto_oto::looks_like_vc_alias(&entry.alias);
                let inferred_vc = crate::app::mvt_auto_oto::looks_like_vc_alias(&entry.alias);
                let (vcv_style, vc_style) = match scheme {
                    crate::app::mvt_auto_oto::MvtOtoScheme::Cv => (false, false),
                    crate::app::mvt_auto_oto::MvtOtoScheme::Vcv => (true, false),
                    crate::app::mvt_auto_oto::MvtOtoScheme::Cvvc => (false, inferred_vc),
                    crate::app::mvt_auto_oto::MvtOtoScheme::Mixed => (inferred_vcv, inferred_vc),
                };
                let generate_vcv_units = segmented_japanese_stem;
                Some(crate::app::mvt_auto_oto::MvtWorkItem {
                    entry_index: index,
                    path: dir.join(&entry.filename),
                    filename: entry.filename.clone(),
                    alias: entry.alias.clone(),
                    propose: selected_indices.contains(&index),
                    offset_ms: entry.offset,
                    preutter_ms: entry.preutter,
                    vcv_style: vcv_style || generate_vcv_units,
                    vc_style,
                    generate_vcv_units,
                    scheme,
                })
            }).collect()
        };
        let mut settings = self.config.mvt_auto_oto.clone();
        settings.refine_model.clear();
        crate::app::mvt_auto_oto::start_analysis(settings, items, &mut self.mvt_auto_oto);
    }

    pub fn apply_mvt_auto_oto_proposals(&mut self) {
        if self.mvt_auto_oto.source_oto_path.as_ref() != self.cur().oto_path.as_ref() {
            self.mvt_auto_oto.status = "As propostas pertencem a outro oto.ini. Analise novamente a seleção desta aba.".into();
            return;
        }
        let proposals = self.mvt_auto_oto.proposals.clone();
        if !proposals.iter().any(|p| p.selected && (p.values.is_some() || p.proposed_alias.is_some())) { return; }
        self.save_undo_state();
        let (applied, first_added_index) = {
            let tab = self.cur_mut();
            let mut applied = 0usize;
            let mut first_added_index = None;
            for proposal in proposals.iter().filter(|p| p.selected) {
                if proposal.new_entry {
                    let (Some(alias), Some(values)) = (&proposal.proposed_alias, proposal.values) else { continue; };
                    if tab.entries.iter().any(|entry| entry.filename == proposal.filename && entry.alias == *alias) { continue; }
                    let line_index = tab.entries.iter().map(|entry| entry.line_index).max().unwrap_or(0).saturating_add(1);
                    tab.entries.push(crate::oto::OtoEntry {
                        filename: proposal.filename.clone(),
                        alias: alias.clone(),
                        offset: values.offset,
                        consonant: values.consonant,
                        cutoff: values.cutoff,
                        preutter: values.preutter,
                        overlap: values.overlap,
                        line_index,
                        done: false,
                        notes: String::new(),
                    });
                    first_added_index.get_or_insert(tab.entries.len() - 1);
                    applied += 1;
                    continue;
                }
                let Some(entry) = tab.entries.get_mut(proposal.entry_index) else { continue; };
                if entry.filename != proposal.filename || entry.alias != proposal.alias { continue; }
                let mut changed = false;
                if let Some(alias) = &proposal.proposed_alias {
                    if entry.alias != *alias { entry.alias = alias.clone(); changed = true; }
                }
                if let Some(values) = proposal.values {
                    if proposal.apply_fields[0] { entry.offset = values.offset; }
                    if proposal.apply_fields[1] { entry.consonant = values.consonant; }
                    if proposal.apply_fields[2] { entry.cutoff = values.cutoff; }
                    if proposal.apply_fields[3] { entry.preutter = values.preutter; }
                    if proposal.apply_fields[4] { entry.overlap = values.overlap; }
                    changed = true;
                }
                if changed { applied += 1; }
            }
            if applied > 0 { tab.dirty = true; }
            (applied, first_added_index)
        };
        if applied > 0 {
            self.cur_mut().multi_selection.clear();
            self.rebuild_filter();
            if let Some(raw_index) = first_added_index {
                if let Some(filtered_index) = self.cur().filtered.iter().position(|&index| index == raw_index) {
                    self.cur_mut().multi_selection.insert(filtered_index);
                    self.select_filtered(filtered_index);
                }
            }
            let msg = format!("Propostas do mVocalToolkit aplicadas a {applied} entrada(s). Revise e salve o oto.ini.");
            self.ui.toast_manager.success(msg.clone());
            self.ui.status = msg;
        }
    }
}
