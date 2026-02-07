import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import type { ProjectWithDetails } from '@/lib/api';
import { queryKeys } from '@/hooks/queries/queryKeys';

interface MoveTaskData {
  id: number;
  columnId: number;
  sortOrder: number;
  sourceColumnId: number;
  projectId: number;
  destinationIndex: number;
  sourceIndex: number;
}

export function useMoveTask() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (data: MoveTaskData) => {
      await api.tasks.update(data.id, {
        column_id: data.columnId,
        sort_order: data.sortOrder,
      });
    },
    onMutate: async (data) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.projects.detail(data.projectId) });
      
      const previousProject = queryClient.getQueryData<ProjectWithDetails>(
        queryKeys.projects.detail(data.projectId)
      );
      
      if (previousProject) {
        const newProject = { ...previousProject };
        newProject.columns = newProject.columns.map(col => ({
          ...col,
          tasks: [...col.tasks],
        }));
        
        const sourceColumn = newProject.columns.find(col => col.id === data.sourceColumnId);
        const targetColumn = newProject.columns.find(col => col.id === data.columnId);
        
        if (sourceColumn && targetColumn) {
          const [movedTask] = sourceColumn.tasks.splice(data.sourceIndex, 1);
          if (movedTask) {
            movedTask.column_id = data.columnId;
            movedTask.sort_order = data.sortOrder;
            targetColumn.tasks.splice(data.destinationIndex, 0, movedTask);
          }
        }
        
        queryClient.setQueryData(queryKeys.projects.detail(data.projectId), newProject);
      }
      
      return { previousProject };
    },
    onError: (_err, data, context) => {
      if (context?.previousProject) {
        queryClient.setQueryData(
          queryKeys.projects.detail(data.projectId),
          context.previousProject
        );
      }
    },
  });
}
