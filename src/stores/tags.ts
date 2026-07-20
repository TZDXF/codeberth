import { ref } from "vue";
import { defineStore } from "pinia";
import { cmd } from "@/lib/tauri";
import type { Tag } from "@/types";

export const useTagsStore = defineStore("tags", () => {
  const tags = ref<Tag[]>([]);

  async function fetchTags() {
    tags.value = await cmd<Tag[]>("list_tags");
  }

  async function createTag(name: string, color: string) {
    const tag = await cmd<Tag>("create_tag", { name, color });
    await fetchTags();
    return tag;
  }

  async function updateTag(id: number, name: string, color: string) {
    const tag = await cmd<Tag>("update_tag", { id, name, color });
    await fetchTags();
    return tag;
  }

  async function deleteTag(id: number) {
    await cmd("delete_tag", { id });
    await fetchTags();
  }

  async function setProjectTags(projectId: number, tagIds: number[]) {
    await cmd("set_project_tags", { projectId, tagIds });
  }

  return { tags, fetchTags, createTag, updateTag, deleteTag, setProjectTags };
});
