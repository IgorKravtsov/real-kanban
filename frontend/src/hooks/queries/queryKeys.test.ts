import { describe, it, expect } from 'vitest'
import { queryKeys } from './queryKeys'

describe('queryKeys', () => {
  describe('projects', () => {
    it('returns correct all key', () => {
      expect(queryKeys.projects.all).toEqual(['projects'])
    })

    it('returns correct lists key', () => {
      expect(queryKeys.projects.lists()).toEqual(['projects', 'list'])
    })

    it('returns correct list key', () => {
      expect(queryKeys.projects.list()).toEqual(['projects', 'list'])
    })

    it('returns correct details key', () => {
      expect(queryKeys.projects.details()).toEqual(['projects', 'detail'])
    })

    it('returns correct detail key with id', () => {
      expect(queryKeys.projects.detail(1)).toEqual(['projects', 'detail', 1])
    })
  })

  describe('tasks', () => {
    it('returns correct all key', () => {
      expect(queryKeys.tasks.all).toEqual(['tasks'])
    })

    it('returns correct lists key', () => {
      expect(queryKeys.tasks.lists()).toEqual(['tasks', 'list'])
    })

    it('returns correct list key without projectId', () => {
      expect(queryKeys.tasks.list()).toEqual(['tasks', 'list'])
    })

    it('returns correct list key with projectId', () => {
      expect(queryKeys.tasks.list(1)).toEqual(['tasks', 'list', { projectId: 1 }])
    })

    it('returns correct details key', () => {
      expect(queryKeys.tasks.details()).toEqual(['tasks', 'detail'])
    })

    it('returns correct detail key with id', () => {
      expect(queryKeys.tasks.detail(5)).toEqual(['tasks', 'detail', 5])
    })
  })

  describe('subtasks', () => {
    it('returns correct all key', () => {
      expect(queryKeys.subtasks.all).toEqual(['subtasks'])
    })

    it('returns correct lists key', () => {
      expect(queryKeys.subtasks.lists()).toEqual(['subtasks', 'list'])
    })

    it('returns correct list key with taskId', () => {
      expect(queryKeys.subtasks.list(3)).toEqual(['subtasks', 'list', { taskId: 3 }])
    })
  })

  describe('tags', () => {
    it('returns correct all key', () => {
      expect(queryKeys.tags.all).toEqual(['tags'])
    })

    it('returns correct lists key', () => {
      expect(queryKeys.tags.lists()).toEqual(['tags', 'list'])
    })

    it('returns correct list key', () => {
      expect(queryKeys.tags.list()).toEqual(['tags', 'list'])
    })
  })
})
