import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Skill } from '@/types';

export const useSkillStore = defineStore('skill', () => {
  const skills = ref<Skill[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchSkills(softwareId: string = '') {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<Skill[]>('get_skills', { softwareId });
      skills.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch skills';
    } finally {
      isLoading.value = false;
    }
  }

  async function createSkill(skill: Partial<Skill> & { softwareId: string; name: string; type: string }) {
    try {
      isLoading.value = true;
      error.value = null;
      const now = new Date().toISOString();
      const newSkill: Skill = {
        id: crypto.randomUUID(),
        softwareId: skill.softwareId,
        name: skill.name,
        type: skill.type,
        config: skill.config || '{}',
        filePath: skill.filePath || '',
        installedAt: now,
      };
      await invoke('create_skill', { skill: newSkill });
      skills.value.push(newSkill);
      return newSkill;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create skill';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function updateSkill(skill: Skill) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('update_skill', { skill });
      const index = skills.value.findIndex(s => s.id === skill.id);
      if (index !== -1) {
        skills.value[index] = skill;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update skill';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteSkill(skillId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('delete_skill', { skillId });
      skills.value = skills.value.filter(s => s.id !== skillId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete skill';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  function getSkillsBySoftware(softwareId: string): Skill[] {
    return skills.value.filter(s => s.softwareId === softwareId);
  }

  function getSkillsByType(type: string): Skill[] {
    return skills.value.filter(s => s.type === type);
  }

  return {
    skills,
    isLoading,
    error,
    fetchSkills,
    createSkill,
    updateSkill,
    deleteSkill,
    getSkillsBySoftware,
    getSkillsByType,
  };
});
