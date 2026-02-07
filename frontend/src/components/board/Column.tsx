import { Droppable } from '@hello-pangea/dnd';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { TaskCard } from './TaskCard';
import type { Task } from '@/lib/api';

interface ColumnProps {
  id: number;
  name: string;
  tasks: Task[];
}

export function Column({ id, name, tasks }: ColumnProps) {
  return (
    <Droppable droppableId={String(id)}>
      {(provided) => (
        <Card 
          className="min-w-[300px] max-w-[300px] bg-muted/50"
          ref={provided.innerRef}
          {...provided.droppableProps}
        >
          <CardHeader className="p-3">
            <CardTitle className="text-sm font-medium">
              {name} ({tasks.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="p-3 pt-0">
            <div className="flex flex-col gap-2">
              {tasks.map((task, index) => (
                <TaskCard key={task.id} task={task} index={index} />
              ))}
              {provided.placeholder}
            </div>
          </CardContent>
        </Card>
      )}
    </Droppable>
  );
}
