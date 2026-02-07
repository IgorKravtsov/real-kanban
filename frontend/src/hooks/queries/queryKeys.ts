export const queryKeys = {
  projects: {
    all: ['projects'] as const,
    lists: () => [...queryKeys.projects.all, 'list'] as const,
    list: () => [...queryKeys.projects.lists()] as const,
    details: () => [...queryKeys.projects.all, 'detail'] as const,
    detail: (id: number) => [...queryKeys.projects.details(), id] as const,
  },
  tasks: {
    all: ['tasks'] as const,
    lists: () => [...queryKeys.tasks.all, 'list'] as const,
    list: (projectId?: number) =>
      projectId
        ? ([...queryKeys.tasks.lists(), { projectId }] as const)
        : ([...queryKeys.tasks.lists()] as const),
    details: () => [...queryKeys.tasks.all, 'detail'] as const,
    detail: (id: number) => [...queryKeys.tasks.details(), id] as const,
  },
  subtasks: {
    all: ['subtasks'] as const,
    lists: () => [...queryKeys.subtasks.all, 'list'] as const,
    list: (taskId: number) =>
      [...queryKeys.subtasks.lists(), { taskId }] as const,
  },
  tags: {
    all: ['tags'] as const,
    lists: () => [...queryKeys.tags.all, 'list'] as const,
    list: () => [...queryKeys.tags.lists()] as const,
  },
};
