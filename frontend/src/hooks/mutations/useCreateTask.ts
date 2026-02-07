import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { queryKeys } from '@/hooks/queries/queryKeys';

interface CreateTaskData {
  projectId: number;
  columnId: number;
  title: string;
}

export function useCreateTask() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (data: CreateTaskData) => {
      return api.tasks.create({
        project_id: data.projectId,
        column_id: data.columnId,
        title: data.title,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    },
  });
}
