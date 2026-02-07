import { Draggable } from '@hello-pangea/dnd';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { Task } from '@/lib/api';

interface TaskCardProps {
  task: Task;
  index: number;
  onClick?: (taskId: number) => void;
}

const priorityColors: Record<string, string> = {
  urgent: 'bg-red-500',
  high: 'bg-orange-500',
  medium: 'bg-blue-500',
  low: 'bg-gray-400',
};

const sourceTagColors: Record<string, string> = {
  cli: 'bg-purple-100 text-purple-800',
  manual: 'bg-green-100 text-green-800',
  ai: 'bg-blue-100 text-blue-800',
};

const sourceTagLabels: Record<string, string> = {
  cli: 'CLI',
  manual: 'Manual',
  ai: 'AI',
};

function truncateDescription(description: string | null | undefined, maxLength = 80): string {
  if (!description) return '';
  if (description.length <= maxLength) return description;
  return description.slice(0, maxLength).trim() + '...';
}

export function TaskCard({ task, index, onClick }: TaskCardProps) {
  const descriptionPreview = truncateDescription(task.description);

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onClick?.(task.id);
  };

  return (
    <Draggable draggableId={String(task.id)} index={index}>
      {(provided) => (
        <div
          ref={provided.innerRef}
          {...provided.draggableProps}
          {...provided.dragHandleProps}
          onClick={handleClick}
        >
          <Card className="p-3 cursor-pointer hover:shadow-md transition-shadow">
            <div className="flex items-start gap-2">
              {task.priority && priorityColors[task.priority] && (
                <div 
                  className={`w-2 h-2 rounded-full mt-1.5 flex-shrink-0 ${priorityColors[task.priority]}`}
                  title={`Priority: ${task.priority}`}
                />
              )}
              <p className="text-sm font-medium flex-1">{task.title}</p>
            </div>
            
            {descriptionPreview && (
              <p className="text-xs text-muted-foreground mt-1.5 line-clamp-2">
                {descriptionPreview}
              </p>
            )}
            
            {task.source_tag && (
              <div className="mt-2">
                <Badge 
                  variant="secondary" 
                  className={`text-xs ${sourceTagColors[task.source_tag] || 'bg-gray-100 text-gray-800'}`}
                >
                  {sourceTagLabels[task.source_tag] || task.source_tag}
                </Badge>
              </div>
            )}
          </Card>
        </div>
      )}
    </Draggable>
  );
}
