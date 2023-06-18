use crate::localization::UpdateAllStringsFilesForStringTableEvent;
use crate::prelude::*;
use crate::project::{RecompileLoadedYarnFilesEvent, YarnFilesBeingLoaded};
use bevy::prelude::*;
use bevy::utils::HashSet;
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, SystemSet)]
pub(crate) struct LineIdUpdateSystemSet;

pub(crate) fn line_id_generation_plugin(app: &mut App) {
    app.add_systems(
        (
            handle_yarn_file_events
                .pipe(panic_on_err)
                .run_if(in_development.and_then(has_localizations)),
            handle_yarn_file_events_outside_development.run_if(
                resource_exists::<YarnProject>()
                    .and_then(not(in_development.and_then(has_localizations))),
            ),
        )
            .chain()
            .in_set(LineIdUpdateSystemSet)
            .in_set(YarnSlingerSystemSet),
    );
}

fn handle_yarn_file_events_outside_development(
    mut events: EventReader<AssetEvent<YarnFile>>,
    yarn_files_being_loaded: Res<YarnFilesBeingLoaded>,
    project: Res<YarnProject>,
    mut recompile_events: EventWriter<RecompileLoadedYarnFilesEvent>,
) {
    for event in events.iter() {
        let AssetEvent::Modified { handle } = event else {
                continue;
            };
        if !(yarn_files_being_loaded.0.contains(handle) || project.yarn_files.contains(handle)) {
            continue;
        }
        recompile_events.send(RecompileLoadedYarnFilesEvent);
    }
}

fn handle_yarn_file_events(
    mut events: EventReader<AssetEvent<YarnFile>>,
    mut assets: ResMut<Assets<YarnFile>>,
    asset_server: Res<AssetServer>,
    mut recompile_events: EventWriter<RecompileLoadedYarnFilesEvent>,
    yarn_files_being_loaded: Res<YarnFilesBeingLoaded>,
    project: Option<Res<YarnProject>>,
    mut update_strings_files_writer: EventWriter<UpdateAllStringsFilesForStringTableEvent>,
    mut dialogue_runners: Query<&mut DialogueRunner>,
    mut added_tags: Local<HashSet<Handle<YarnFile>>>,
    mut last_recompiled_yarn_file: Local<Option<YarnFile>>,
) -> SystemResult {
    let mut recompilation_needed = false;
    let mut already_handled = HashSet::new();
    for event in events.iter() {
        let (AssetEvent::Created { handle } | AssetEvent::Modified { handle }) = event else {
            continue;
        };
        if already_handled.contains(handle) {
            continue;
        }
        already_handled.insert(handle);
        if !yarn_files_being_loaded.0.contains(handle)
            && !project
                .as_ref()
                .map(|p| p.yarn_files.contains(handle))
                .unwrap_or_default()
        {
            continue;
        }
        let yarn_file = assets.get(handle).unwrap();

        update_strings_files_writer.send(UpdateAllStringsFilesForStringTableEvent(
            yarn_file.string_table.clone(),
        ));

        let Some(source_with_added_ids) = add_tags_to_lines(yarn_file)? else {
            if matches!(event, AssetEvent::Created { .. }) {
                continue;
            }
            if last_recompiled_yarn_file.as_ref() == Some(yarn_file) {
                // Sometimes `Modified` events are sent twice in a row for the same file for some reason...
                continue;
            }
            last_recompiled_yarn_file.replace(yarn_file.clone());
            for mut dialogue_runner in dialogue_runners.iter_mut() {
                dialogue_runner.text_provider.extend_base_string_table(yarn_file.string_table.clone());
            }
            added_tags.remove(handle);
            recompilation_needed = true;
            continue;
        };

        if added_tags.contains(handle) {
            continue;
        }
        let asset_path = asset_server
            .get_handle_path(handle.clone())
            .with_context(|| format!("Failed to overwrite Yarn file \"{}\" with new IDs because it was not found on disk",
                                     yarn_file.file_name()))?;
        let assets_path = get_assets_dir_path(&asset_server)?;
        let path_within_asset_dir: PathBuf =
            [assets_path.as_ref(), asset_path.path()].iter().collect();

        std::fs::write(&path_within_asset_dir, &source_with_added_ids)
                    .context(
                        format!("Failed to overwrite Yarn file at {} with new line IDs. \
                                 Aborting because localization requires all lines to have IDs, but this file is missing some.",
                                path_within_asset_dir.display()))?;

        info!(
            "Automatically generated line IDs for Yarn file at {}",
            path_within_asset_dir.display()
        );
        let is_watching = project
            .as_ref()
            .map(|p| p.watching_for_changes)
            .unwrap_or_default();
        if is_watching {
            added_tags.insert(handle.clone_weak());
        } else {
            let yarn_file = assets.get_mut(handle).unwrap();
            yarn_file.file.source = source_with_added_ids;

            let string_table = YarnCompiler::new()
                .with_compilation_type(CompilationType::StringsOnly)
                .add_file(yarn_file.file.clone())
                .compile()?
                .string_table;
            yarn_file.string_table = string_table;
        }
        // Recompilations is triggered later via another `AssetEvent::Modified`
    }

    if recompilation_needed && project.is_some() {
        recompile_events.send(RecompileLoadedYarnFilesEvent);
    }
    Ok(())
}

/// Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner-Console/blob/main/src/YarnSpinner.Console/Commands/TagCommand.cs#L11>
fn add_tags_to_lines(yarn_file: &YarnFile) -> YarnCompilerResult<Option<String>> {
    let existing_tags = yarn_file
        .string_table
        .iter()
        .filter_map(|(key, string_info)| (!string_info.is_implicit_tag).then(|| key.clone()))
        .collect();
    YarnCompiler::add_tags_to_lines(yarn_file.file.source.clone(), existing_tags)
}
