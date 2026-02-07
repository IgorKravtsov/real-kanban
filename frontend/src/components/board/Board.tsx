import { useState } from 'react';
import { DragDropContext, Droppable, Draggable } from '@hello-pangea/dnd';
import type { DropResult } from '@hello-pangea/dnd';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useMoveTask } from '@/hooks/mutations/useMoveTask';
import { useCreateTask } from '@/hooks/mutations/useCreateTask';
import { useCreateColumn, useUpdateColumn, useDeleteColumn, useReorderColumns } from '@/hooks/mutations/useColumns';
import { TaskDetailModal } from '@/components/TaskDetailModal';
import { TaskCard } from '@/components/board/TaskCard';
import type { ColumnWithTasks } from '@/lib/api';
import { Plus, X, MoreHorizontal, Pencil, Trash2, GripVertical } from 'lucide-react';

interface BoardProps {
  columns: ColumnWithTasks[];
}

export function Board({ columns }: BoardProps) {
  const moveTask = useMoveTask();
  const createTask = useCreateTask();
  const createColumn = useCreateColumn();
  const updateColumn = useUpdateColumn();
  const deleteColumn = useDeleteColumn();
  const reorderColumns = useReorderColumns();
  
  const [addingToColumn, setAddingToColumn] = useState<number | null>(null);
  const [newTaskTitle, setNewTaskTitle] = useState('');
  const [selectedTaskId, setSelectedTaskId] = useState<number | null>(null);
  const [addingColumn, setAddingColumn] = useState(false);
  const [newColumnName, setNewColumnName] = useState('');
  const [editingColumnId, setEditingColumnId] = useState<number | null>(null);
  const [editingColumnName, setEditingColumnName] = useState('');

  const projectId = columns[0]?.project_id;

  const handleDragEnd = (result: DropResult) => {
    if (!result.destination) return;

    if (result.type === 'COLUMN') {
      const sourceIndex = result.source.index;
      const destIndex = result.destination.index;
      
      if (sourceIndex === destIndex) return;
      if (!projectId) return;

      const reorderedColumns = [...columns];
      const [movedColumn] = reorderedColumns.splice(sourceIndex, 1);
      reorderedColumns.splice(destIndex, 0, movedColumn);

      const newOrder = reorderedColumns.map((col, index) => ({
        id: col.id,
        sort_order: (index + 1) * 1000,
      }));

      reorderColumns.mutate({ projectId, items: newOrder });
      return;
    }
    
    const sourceColumnId = Number.parseInt(result.source.droppableId, 10);
    const destColumnId = Number.parseInt(result.destination.droppableId, 10);
    const taskId = Number.parseInt(result.draggableId, 10);
    
    if (sourceColumnId === destColumnId && result.source.index === result.destination.index) {
      return;
    }
    
    if (!projectId) return;
    
    moveTask.mutate({
      id: taskId,
      columnId: destColumnId,
      sortOrder: result.destination.index * 1000,
      sourceColumnId,
      projectId,
      destinationIndex: result.destination.index,
      sourceIndex: result.source.index,
    });
  };

  const handleAddTask = (columnId: number, projectId: number) => {
    if (!newTaskTitle.trim()) return;
    
    createTask.mutate({
      projectId,
      columnId,
      title: newTaskTitle.trim(),
    }, {
      onSuccess: () => {
        setNewTaskTitle('');
        setAddingToColumn(null);
      },
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent, columnId: number, projectId: number) => {
    if (e.key === 'Enter') {
      handleAddTask(columnId, projectId);
    } else if (e.key === 'Escape') {
      setNewTaskTitle('');
      setAddingToColumn(null);
    }
  };

  const handleTaskClick = (taskId: number) => {
    setSelectedTaskId(taskId);
  };

  const handleAddColumn = () => {
    if (!newColumnName.trim() || !projectId) return;
    
    createColumn.mutate({
      projectId,
      name: newColumnName.trim(),
    }, {
      onSuccess: () => {
        setNewColumnName('');
        setAddingColumn(false);
      },
    });
  };

  const handleColumnKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleAddColumn();
    } else if (e.key === 'Escape') {
      setNewColumnName('');
      setAddingColumn(false);
    }
  };

  const handleEditColumn = (column: ColumnWithTasks) => {
    setEditingColumnId(column.id);
    setEditingColumnName(column.name);
  };

  const handleSaveColumnName = () => {
    if (!editingColumnName.trim() || !projectId || !editingColumnId) return;
    
    updateColumn.mutate({
      id: editingColumnId,
      name: editingColumnName.trim(),
      projectId,
    }, {
      onSuccess: () => {
        setEditingColumnId(null);
        setEditingColumnName('');
      },
    });
  };

  const handleEditKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSaveColumnName();
    } else if (e.key === 'Escape') {
      setEditingColumnId(null);
      setEditingColumnName('');
    }
  };

  const handleDeleteColumn = (columnId: number) => {
    if (!projectId) return;
    
    const column = columns.find(c => c.id === columnId);
    const taskCount = column?.tasks.length || 0;
    
    const message = taskCount > 0 
      ? `This column has ${taskCount} task(s). Deleting it will also delete all tasks. Are you sure?`
      : 'Are you sure you want to delete this column?';
    
    if (confirm(message)) {
      deleteColumn.mutate({ id: columnId, projectId });
    }
  };

  return (
    <>
      <DragDropContext onDragEnd={handleDragEnd}>
        <Droppable droppableId="board" type="COLUMN" direction="horizontal">
          {(provided) => (
            <div
              ref={provided.innerRef}
              {...provided.droppableProps}
              className="flex gap-4 overflow-x-auto p-4 h-full"
            >
              {columns.map((column, index) => (
                <Draggable key={column.id} draggableId={`column-${column.id}`} index={index}>
                  {(dragProvided, dragSnapshot) => (
                    <div
                      ref={dragProvided.innerRef}
                      {...dragProvided.draggableProps}
                      className={dragSnapshot.isDragging ? 'opacity-90' : ''}
                    >
                      <Droppable droppableId={String(column.id)} type="TASK">
                        {(dropProvided) => (
                          <Card 
                            className="min-w-[300px] max-w-[300px] bg-muted/50 flex flex-col h-full"
                            ref={dropProvided.innerRef}
                            {...dropProvided.droppableProps}
                          >
                            <CardHeader className="p-3 flex-row items-center justify-between space-y-0">
                              <div
                                {...dragProvided.dragHandleProps}
                                className="cursor-grab text-muted-foreground hover:text-foreground transition-colors mr-1"
                              >
                                <GripVertical className="h-4 w-4" />
                              </div>
                              {editingColumnId === column.id ? (
                                <Input
                                  autoFocus
                                  value={editingColumnName}
                                  onChange={(e) => setEditingColumnName(e.target.value)}
                                  onKeyDown={handleEditKeyDown}
                                  onBlur={handleSaveColumnName}
                                  className="h-7 text-sm font-medium flex-1"
                                />
                              ) : (
                                <CardTitle className="text-sm font-medium flex-1">
                                  {column.name} ({column.tasks.length})
                                </CardTitle>
                              )}
                              <div className="flex items-center gap-1">
                                <Button
                                  variant="ghost"
                                  size="icon"
                                  className="h-6 w-6"
                                  onClick={() => setAddingToColumn(column.id)}
                                >
                                  <Plus className="h-4 w-4" />
                                </Button>
                                <DropdownMenu>
                                  <DropdownMenuTrigger asChild>
                                    <Button variant="ghost" size="icon" className="h-6 w-6">
                                      <MoreHorizontal className="h-4 w-4" />
                                    </Button>
                                  </DropdownMenuTrigger>
                                  <DropdownMenuContent align="end">
                                    <DropdownMenuItem onClick={() => handleEditColumn(column)}>
                                      <Pencil className="h-4 w-4 mr-2" />
                                      Rename
                                    </DropdownMenuItem>
                                    <DropdownMenuItem 
                                      onClick={() => handleDeleteColumn(column.id)}
                                      className="text-destructive focus:text-destructive"
                                    >
                                      <Trash2 className="h-4 w-4 mr-2" />
                                      Delete
                                    </DropdownMenuItem>
                                  </DropdownMenuContent>
                                </DropdownMenu>
                              </div>
                            </CardHeader>
                            <CardContent className="p-3 pt-0 flex-1 overflow-y-auto">
                              <div className="flex flex-col gap-2">
                                {addingToColumn === column.id && (
                                  <Card className="p-2">
                                    <Input
                                      autoFocus
                                      placeholder="Task title..."
                                      value={newTaskTitle}
                                      onChange={(e) => setNewTaskTitle(e.target.value)}
                                      onKeyDown={(e) => handleKeyDown(e, column.id, column.project_id)}
                                      className="mb-2"
                                    />
                                    <div className="flex gap-2">
                                      <Button
                                        size="sm"
                                        onClick={() => handleAddTask(column.id, column.project_id)}
                                        disabled={!newTaskTitle.trim() || createTask.isPending}
                                      >
                                        Add
                                      </Button>
                                      <Button
                                        size="sm"
                                        variant="ghost"
                                        onClick={() => {
                                          setNewTaskTitle('');
                                          setAddingToColumn(null);
                                        }}
                                      >
                                        <X className="h-4 w-4" />
                                      </Button>
                                    </div>
                                  </Card>
                                )}
                                {column.tasks.map((task, index) => (
                                  <TaskCard
                                    key={task.id}
                                    task={task}
                                    index={index}
                                    onClick={handleTaskClick}
                                  />
                                ))}
                                {dropProvided.placeholder}
                              </div>
                            </CardContent>
                          </Card>
                        )}
                      </Droppable>
                    </div>
                  )}
                </Draggable>
              ))}
              {provided.placeholder}
              
              {addingColumn ? (
                <Card className="min-w-[300px] max-w-[300px] bg-muted/50 p-3">
                  <Input
                    autoFocus
                    placeholder="Column name..."
                    value={newColumnName}
                    onChange={(e) => setNewColumnName(e.target.value)}
                    onKeyDown={handleColumnKeyDown}
                    className="mb-2"
                  />
                  <div className="flex gap-2">
                    <Button
                      size="sm"
                      onClick={handleAddColumn}
                      disabled={!newColumnName.trim() || createColumn.isPending}
                    >
                      Add Column
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        setNewColumnName('');
                        setAddingColumn(false);
                      }}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                </Card>
              ) : (
                <Button
                  variant="outline"
                  className="min-w-[300px] h-auto py-6 border-dashed"
                  onClick={() => setAddingColumn(true)}
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Add Column
                </Button>
              )}
            </div>
          )}
        </Droppable>
      </DragDropContext>

      <TaskDetailModal
        taskId={selectedTaskId}
        isOpen={selectedTaskId !== null}
        onClose={() => setSelectedTaskId(null)}
      />
    </>
  );
}
