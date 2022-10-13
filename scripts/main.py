from typing import List, Optional, Set, TextIO, Tuple
import sys
import glob
import os
from venv import create
from graph_tool.all import *


def read_graph(file: TextIO, solution: Optional[Set[int]] = None) -> Tuple[Graph, VertexPropertyMap, VertexPropertyMap]:
    g = Graph()
    colors = None
    sizes = None
    colors = g.new_vertex_property('vector<double>')
    sizes = g.new_vertex_property('int')

    first = file.readline()
    parameters = first.strip().split(' ')
    vertices = int(parameters[0])

    vertex_list: List[Vertex] = []
    for i in range(vertices):
        v = g.add_vertex()
        if solution != None and i in solution:
            colors[v] = [1., 165/255, 0, 0.8]
            sizes[v] = 5
        else:
            colors[v] = [0., 1., 1., 0.8]
            sizes[v] = 2
        vertex_list.append(v)

    vertex = -1
    for line in file.readlines():
        vertex += 1
        if line.strip() == '':
            continue
        edge_list = [(vertex_list[vertex], vertex_list[int(x) - 1])
                     for x in line.strip().split(' ')]
        g.add_edge_list(edge_list)

    return g, colors, sizes


def main():
    directory = '../kernels'
    solution: Optional[Set[int]] = None

    if len(sys.argv) >= 2:
        directory = sys.argv[1]

    # solution = set()
    # with open('./solutions/se_059') as sol_file:
    #     for line in sol_file.readlines():
    #         vertex = int(line.strip())
    #         solution.add(vertex)

    for path in glob.glob(f'./{directory}/*'):
        filename = os.path.basename(path)
        with open(path) as file:
            print(path)
            graph, colors, sizes = read_graph(file, solution)
            try:
                pos = sfdp_layout(graph)
                graph_draw(graph, pos=pos, vertex_fill_color=colors, vertex_size=sizes, vertex_text=graph.vertex_index,
                        output=f'../graphs/{filename}-vis.pdf')
            except Exception:
                print(f'{path} had an exception')


if __name__ == '__main__':
    main()
