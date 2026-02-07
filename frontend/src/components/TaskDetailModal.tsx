import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Loader2, Pencil } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { queryKeys } from '@/hooks/queries/queryKeys';
import { LinkifyText } from '@/components/ui/linkify-text';

interface TaskDetailModalProps {
  taskId: number | null;
  isOpen: boolean;
  onClose: () => void;
}

const priorityColors: Record<string, string> = {
  urgent: 'bg-red-500',
  high: 'bg-orange-500',
  medium: 'bg-blue-500',
  low: 'bg-gray-400',
};

const priorities = [
  { value: 'urgent', label: 'Urgent', color: 'bg-red-500' },
  { value: 'high', label: 'High', color: 'bg-orange-500' },
  { value: 'medium', label: 'Medium', color: 'bg-blue-500' },
  { value: 'low', label: 'Low', color: 'bg-gray-400' },
];

export function TaskDetailModal({ taskId, isOpen, onClose }: TaskDetailModalProps) {
  const queryClient = useQueryClient();
  
  const { data: task, isLoading } = useQuery({
    queryKey: ['task', taskId],
    queryFn: () => api.tasks.get(taskId!),
    enabled: !!taskId && isOpen,
  });

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<string>('');
  const [isEditingDescription, setIsEditingDescription] = useState(false);
  useEffect(() => {
    if (task) {
      setTitle(task.title);
      setDescription(task.description || '');
      setPriority(task.priority || '');
      setIsEditingDescription(false);
    }
  }, [task]);

  const updateTask = useMutation({
    mutationFn: (data: { title?: string; description?: string; priority?: string }) =>
      api.tasks.update(taskId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
      queryClient.invalidateQueries({ queryKey: ['task', taskId] });
      onClose();
    },
  });

  const deleteTask = useMutation({
    mutationFn: () => api.tasks.delete(taskId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
      onClose();
    },
  });

  const handleSave = () => {
    updateTask.mutate({ 
      title, 
      description,
      priority: priority || undefined,
    });
  };

  const handleDelete = () => {
    if (confirm('Are you sure you want to delete this task?')) {
      deleteTask.mutate();
    }
  };

  if (!taskId) return null;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl h-[80vh] flex flex-col">
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <Loader2 className="h-8 w-8 animate-spin" />
          </div>
        ) : task ? (
          <>
            <DialogHeader className="flex-shrink-0">
              <DialogTitle className="flex items-center gap-2">
                <Input 
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  className="flex-1 text-lg font-semibold border-none shadow-none focus-visible:ring-0 px-0"
                  placeholder="Task title"
                />
              </DialogTitle>
            </DialogHeader>

            <div className="flex-1 overflow-y-auto space-y-6 py-4">
              <div className="flex items-center gap-4 flex-wrap">
                <div className="space-y-1">
                  <label className="text-sm font-medium">Priority</label>
                  <Select value={priority} onValueChange={setPriority}>
                    <SelectTrigger className="w-[140px]">
                      <SelectValue placeholder="Set priority">
                        {priority && (
                          <div className="flex items-center gap-2">
                            <div className={`w-2 h-2 rounded-full ${priorityColors[priority]}`} />
                            {priorities.find(p => p.value === priority)?.label}
                          </div>
                        )}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {priorities.map((p) => (
                        <SelectItem key={p.value} value={p.value}>
                          <div className="flex items-center gap-2">
                            <div className={`w-2 h-2 rounded-full ${p.color}`} />
                            {p.label}
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                {task.source_tag && (
                  <Badge variant="outline" className="h-fit">
                    Source: {task.source_tag}
                  </Badge>
                )}
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <label className="text-sm font-medium">Description</label>
                  {!isEditingDescription && description && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setIsEditingDescription(true)}
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                  )}
                </div>
                {isEditingDescription || !description ? (
                  <Textarea
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    onFocus={() => setIsEditingDescription(true)}
                    placeholder="Add a description..."
                    className="min-h-[200px] resize-none"
                  />
                ) : (
                  <div
                    className="min-h-[200px] p-3 rounded-md border bg-muted/50 whitespace-pre-wrap cursor-text"
                    onClick={() => setIsEditingDescription(true)}
                  >
                    <LinkifyText text={description} />
                  </div>
                )}
              </div>
            </div>

            <div className="flex-shrink-0 flex justify-between gap-2 pt-4 border-t">
              <Button 
                variant="destructive" 
                onClick={handleDelete}
                disabled={deleteTask.isPending}
              >
                {deleteTask.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  'Delete'
                )}
              </Button>
              <div className="flex gap-2">
                <Button variant="outline" onClick={onClose}>
                  Cancel
                </Button>
                <Button 
                  onClick={handleSave}
                  disabled={updateTask.isPending}
                >
                  {updateTask.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    'Save Changes'
                  )}
                </Button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-muted-foreground">
            Task not found
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
