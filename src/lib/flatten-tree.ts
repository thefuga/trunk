import type { TreeNode, DirectoryNode, FileNode } from './build-tree.js';

export interface FlatFileRow {
  type: 'file';
  depth: number;
  node: FileNode;
  parentPath: string | null;
}

export interface FlatDirRow {
  type: 'directory';
  depth: number;
  node: DirectoryNode;
  expanded: boolean;
  parentPath: string | null;
}

export type FlatRow = FlatFileRow | FlatDirRow;

/**
 * Flatten a nested TreeNode[] into a flat array of rows, honoring the expanded set.
 *
 * - Collapsed directories appear as a single row with expanded: false
 * - Expanded directories show their children at depth + 1
 * - Order is preserved from the input (directories before files per buildTree)
 */
export function flattenTree(
  nodes: TreeNode[],
  expanded: Set<string>,
  depth: number = 0,
  parentPath: string | null = null,
): FlatRow[] {
  const result: FlatRow[] = [];
  for (const node of nodes) {
    if (node.type === 'directory') {
      const isExpanded = expanded.has(node.path);
      result.push({ type: 'directory', depth, node, expanded: isExpanded, parentPath });
      if (isExpanded) {
        result.push(...flattenTree(node.children, expanded, depth + 1, node.path));
      }
    } else {
      result.push({ type: 'file', depth, node, parentPath });
    }
  }
  return result;
}

/**
 * Find the index of a row matching a given path, or 0 if not found.
 */
export function findFocusIndex(rows: FlatRow[], path: string): number {
  if (rows.length === 0) return 0;
  const idx = rows.findIndex(r => r.node.path === path);
  return idx >= 0 ? idx : 0;
}
