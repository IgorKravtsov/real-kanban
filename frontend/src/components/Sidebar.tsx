import { Link, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { DragDropContext, Droppable, Draggable } from '@hello-pangea/dnd';
import type { DropResult } from '@hello-pangea/dnd';
import { api } from '@/lib/api';
import type { Project } from '@/lib/api';
import { cn } from '@/lib/utils';
import { FolderKanban, Plus, Home, GripVertical, PanelLeftClose, PanelLeft } from 'lucide-react';
import { ThemeToggle } from '@/components/theme-toggle';
import { Button } from '@/components/ui/button';

interface SidebarProps {
  isCollapsed: boolean;
  onToggle: () => void;
}

export function Sidebar({ isCollapsed, onToggle }: SidebarProps) {
  const { id } = useParams<{ id: string }>();
  const currentProjectId = id ? Number.parseInt(id, 10) : null;
  const queryClient = useQueryClient();

  const { data: projects } = useQuery({
    queryKey: ['projects'],
    queryFn: api.projects.list,
  });

  const reorderMutation = useMutation({
    mutationFn: api.projects.reorder,
    onMutate: async (newOrder) => {
      await queryClient.cancelQueries({ queryKey: ['projects'] });
      const previousProjects = queryClient.getQueryData<Project[]>(['projects']);
      
      if (previousProjects) {
        const reorderedProjects = [...previousProjects].sort((a, b) => {
          const aOrder = newOrder.find(item => item.id === a.id)?.sort_order ?? a.sort_order;
          const bOrder = newOrder.find(item => item.id === b.id)?.sort_order ?? b.sort_order;
          return aOrder - bOrder;
        });
        queryClient.setQueryData(['projects'], reorderedProjects);
      }
      
      return { previousProjects };
    },
    onError: (_err, _newOrder, context) => {
      if (context?.previousProjects) {
        queryClient.setQueryData(['projects'], context.previousProjects);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });

  const handleDragEnd = (result: DropResult) => {
    if (!result.destination || !projects) return;
    
    const sourceIndex = result.source.index;
    const destIndex = result.destination.index;
    
    if (sourceIndex === destIndex) return;

    const reorderedProjects = [...projects];
    const [movedProject] = reorderedProjects.splice(sourceIndex, 1);
    reorderedProjects.splice(destIndex, 0, movedProject);

    const newOrder = reorderedProjects.map((project, index) => ({
      id: project.id,
      sort_order: (index + 1) * 1000,
    }));

    reorderMutation.mutate(newOrder);
  };

  if (isCollapsed) {
    return (
      <aside className="w-14 border-r bg-muted/30 flex flex-col h-full">
        <div className="p-2 border-b flex justify-center">
          <Button variant="ghost" size="icon" onClick={onToggle} title="Expand sidebar">
            <PanelLeft className="h-5 w-5" />
          </Button>
        </div>

        <nav className="flex-1 overflow-y-auto p-2 flex flex-col items-center gap-1">
          <Link
            to="/"
            className={cn(
              'flex items-center justify-center p-2 rounded-md transition-colors',
              currentProjectId === null
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted'
            )}
            title="All Projects"
          >
            <Home className="h-4 w-4" />
          </Link>

          <div className="w-full h-px bg-border my-2" />

          {projects?.map((project) => (
            <Link
              key={project.id}
              to={`/projects/${project.id}`}
              className={cn(
                'flex items-center justify-center w-8 h-8 rounded-md transition-colors text-xs font-medium',
                currentProjectId === project.id
                  ? 'bg-primary text-primary-foreground'
                  : 'hover:bg-muted'
              )}
              title={project.name}
            >
              {project.name.charAt(0).toUpperCase()}
            </Link>
          ))}
        </nav>

        <div className="p-2 border-t flex justify-center">
          <ThemeToggle />
        </div>

        <div className="p-2 border-t flex justify-center">
          <Link
            to="/"
            className="flex items-center justify-center p-2 rounded-md hover:bg-muted transition-colors"
            title="New Project"
          >
            <Plus className="h-4 w-4" />
          </Link>
        </div>
      </aside>
    );
  }

  return (
    <aside className="w-64 border-r bg-muted/30 flex flex-col h-full">
      <div className="p-4 border-b flex items-center justify-between">
        <Link to="/" className="flex items-center gap-2 font-semibold text-lg">
          <FolderKanban className="h-5 w-5" />
          <span>Real Kanban</span>
        </Link>
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onToggle} title="Collapse sidebar">
          <PanelLeftClose className="h-4 w-4" />
        </Button>
      </div>

      <nav className="flex-1 overflow-y-auto p-2">
        <div className="mb-2">
          <Link
            to="/"
            className={cn(
              'flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors',
              currentProjectId === null
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted'
            )}
          >
            <Home className="h-4 w-4" />
            <span>All Projects</span>
          </Link>
        </div>

        <div className="mt-4">
          <div className="px-3 py-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Projects
          </div>
          <DragDropContext onDragEnd={handleDragEnd}>
            <Droppable droppableId="projects">
              {(provided) => (
                <div
                  ref={provided.innerRef}
                  {...provided.droppableProps}
                  className="mt-1 space-y-1"
                >
                  {projects?.map((project, index) => (
                    <Draggable
                      key={project.id}
                      draggableId={String(project.id)}
                      index={index}
                    >
                      {(provided, snapshot) => (
                        <div
                          ref={provided.innerRef}
                          {...provided.draggableProps}
                          className={cn(
                            'flex items-center gap-1 rounded-md text-sm transition-colors group',
                            currentProjectId === project.id
                              ? 'bg-primary text-primary-foreground'
                              : 'hover:bg-muted',
                            snapshot.isDragging && 'bg-muted shadow-lg'
                          )}
                        >
                          <div
                            {...provided.dragHandleProps}
                            className={cn(
                              'p-1 cursor-grab opacity-0 group-hover:opacity-100 transition-opacity',
                              currentProjectId === project.id
                                ? 'text-primary-foreground/70'
                                : 'text-muted-foreground'
                            )}
                          >
                            <GripVertical className="h-4 w-4" />
                          </div>
                          <Link
                            to={`/projects/${project.id}`}
                            className="flex items-center gap-2 px-2 py-2 flex-1 min-w-0"
                          >
                            <FolderKanban className="h-4 w-4 shrink-0" />
                            <span className="truncate">{project.name}</span>
                          </Link>
                        </div>
                      )}
                    </Draggable>
                  ))}
                  {provided.placeholder}
                </div>
              )}
            </Droppable>
          </DragDropContext>
        </div>
      </nav>

      <div className="p-2 border-t flex items-center justify-between">
        <ThemeToggle />
        <Button variant="ghost" size="sm" className="flex-1 justify-start ml-2" asChild>
          <Link to="/">
            <Plus className="h-4 w-4 mr-2" />
            New Project
          </Link>
        </Button>
      </div>
    </aside>
  );
}
