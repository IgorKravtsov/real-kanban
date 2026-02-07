import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Board } from '@/components/board/Board';
import { useProject } from '@/hooks/queries/useProject';
import { Loader2, FolderOpen, Monitor } from 'lucide-react';
import { api } from '@/lib/api';

export default function ProjectBoardPage() {
  const { id } = useParams<{ id: string }>();
  const projectId = id ? Number.parseInt(id, 10) : 0;
  const { data: project, isLoading, error } = useProject(projectId);
  
  const { data: linkedPaths } = useQuery({
    queryKey: ['linkedPaths', projectId],
    queryFn: () => api.linkedPaths.list(projectId),
    enabled: !!projectId,
  });

  if (!id) {
    return (
      <div className="p-8">
        <h1 className="text-2xl font-bold mb-4">Error</h1>
        <p className="text-muted-foreground">Invalid project ID</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin" />
      </div>
    );
  }

  if (error || !project) {
    return (
      <div className="p-8">
        <h1 className="text-2xl font-bold mb-4">Error</h1>
        <p className="text-muted-foreground">Failed to load project</p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <header className="border-b p-4 flex items-center gap-4">
        <div className="flex-1">
          <h1 className="text-xl font-semibold">{project.name}</h1>
          <p className="text-sm text-muted-foreground">
            {project.columns?.length || 0} columns
          </p>
        </div>
        
        {linkedPaths && linkedPaths.length > 0 && (
          <div className="flex items-center gap-2">
            <FolderOpen className="h-4 w-4 text-muted-foreground" />
            <div className="flex flex-wrap gap-2">
              {linkedPaths.map((lp) => (
                <div
                  key={lp.id}
                  className="flex items-center gap-1.5 px-2 py-1 bg-muted rounded-md text-xs"
                  title={lp.path}
                >
                  {lp.hostname && (
                    <>
                      <Monitor className="h-3 w-3 text-muted-foreground" />
                      <span className="text-muted-foreground">{lp.hostname}:</span>
                    </>
                  )}
                  <span 
                    className="font-mono max-w-[200px] truncate block"
                    style={{ direction: 'rtl', textAlign: 'left' }}
                  >{lp.path}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </header>
      
      <main className="flex-1 overflow-hidden">
        <Board columns={project.columns || []} />
      </main>
    </div>
  );
}
