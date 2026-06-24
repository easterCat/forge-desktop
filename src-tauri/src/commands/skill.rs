use crate::models::Skill;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_skills(state: State<AppState>, software_id: String) -> Result<Vec<Skill>, String> {
    log::info!("Getting skills for software: {}", software_id);
    state.db.get_skills_by_software(&software_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_skill(state: State<AppState>, skill: Skill) -> Result<(), String> {
    log::info!("Creating skill: {}", skill.name);
    state.db.upsert_skill(&skill).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_skill(state: State<AppState>, skill: Skill) -> Result<(), String> {
    log::info!("Updating skill: {}", skill.name);
    state.db.upsert_skill(&skill).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_skill(state: State<AppState>, skill_id: String) -> Result<(), String> {
    log::info!("Deleting skill: {}", skill_id);
    state.db.delete_skill(&skill_id).map_err(|e| e.to_string())
}
