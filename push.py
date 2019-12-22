import math
import operator
import random
import sys

import numpy as np

# Wikipedia: 34 degrees
ANGLE_OF_REPOSE_SAND = 34.0 / math.pi / 2 * 360.0
CRITICAL_SLOPE = math.tan(ANGLE_OF_REPOSE_SAND)
print(CRITICAL_SLOPE)
#GRAIN_INTERLOCK = (0, 0, 1, 1, 1, 1, 2, 2)
#GRAIN_INTERLOCK = (0, 0, 1, 1, 1, 1, 2)
GRAIN_INTERLOCK = (0, 1, 1, 1, 1, 1, 1, 1, 1)

def mkarena(size:int, level:int=20):
    return np.full((size, size), level, np.int)

def dist2d(a, b):
    return ((a[0] - b[0])**2 + (a[1] - b[1])**2)**0.5

def mkball(r:int=12, level=4):
    d = r*2
    rsq = r*r
    heightmap = np.zeros((d+1, d+1))
    from math import sqrt
    def _hemisphere(x, y):
        # (x - a)² + (y - b)² + (z - c)² = r²
        # z = sqrt( r^2 - x^2 - y^2)
        xt = x-r
        yt = y-r
        v = xt*xt + yt*yt
        if abs(v) <= rsq:
            return sqrt(v)
        return 255
    def mask(x, y):
        xt = x-r
        yt = y-r
        v = xt*xt + yt*yt
        return v < rsq
        
    return (np.fromfunction(np.vectorize(_hemisphere), (d+1, d+1), dtype=np.int),
            np.fromfunction(mask, (d+1, d+1), dtype=np.int))
 

def centerdistpush(shape):
    c0 = shape[0]/2
    c1 = shape[1]/2
    key=lambda i0, i1: np.sqrt(np.square(i0 - c0) + np.square(i1 - c1))
    dist = np.fromfunction(key, shape, dtype=np.int).astype(np.int)

    # Direction to push overflow grains at this location. (Because the ball is
    # a sphere)
    dx2 = np.fromfunction(np.vectorize(lambda i0, i1: pushdir((c0, c1), (i0, i1))[0]), shape, dtype=np.int).astype(np.int)
    dy2 = np.fromfunction(np.vectorize(lambda i0, i1: pushdir((c0, c1), (i0, i1))[1]), shape, dtype=np.int).astype(np.int)
    dx, dy = pushdir_array(shape, (c0, c1))
    return dist, dx, dy, dx2, dy2

def pushdir_array(shape, pos):
    c0 = pos[0]
    c1 = pos[1]
    x = np.fromfunction(lambda i0, i1: i0-c0, shape).astype(np.float)
    y = np.fromfunction(lambda i0, i1: i1-c1, shape).astype(np.float)
    r = x / y
    ar = np.absolute(r)
    a = 2.
    inva = 1/a
    x[ar < inva] = 0
    y[ar > a] = 0
    dx = np.sign(x).astype(np.int)
    dy = np.sign(y).astype(np.int)
    return dx, dy


def pushdir(pos, i):
    x = i[0] - pos[0]
    y = i[1] - pos[1]
    if y == 0:
        r = x
    else:
        r = x / y
    ar = abs(r)
    if ar < 0.5:
        x = 0
    if ar > 2.0:
        y = 0
    return (int(np.sign(x)), int(np.sign(y)))

LAND = ".123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWYZ"
LAND = LAND + LAND[1:]
LAND = LAND + LAND[1:]
LAND = LAND + LAND[1:]
r = 12
level = 6
def render(field, glyphmap=LAND, overflow='*'):
    def toglyph():
        for x in range (0, field.shape[0]):
            for y in range (0, field.shape[1]):
                if field[x][y] < len(glyphmap):
                    yield glyphmap[field[x][y]]
                else:
                    yield overflow
            yield '\n'
    return ''.join(toglyph())

def prender(*fields, glyphmap=LAND):
    lines = []
    for field in fields:
        lines.append(render(field, glyphmap).split('\n'))
    for rows in zip(*lines):
        print('  '.join(rows))


def find_corners(pos, s2, bounds):
    corners =  ((pos[0] - s2[0]//2, pos[1]- s2[1]//2),
            (pos[0] + s2[0]//2+1, pos[1] + s2[1]//2+1))
    b = None
    if (corners[0][0]< 0
            or corners[0][1] < 0
            or corners[1][0] > bounds[0]
            or corners[1][1] > bounds[1]):
        b = True
        b00 = 0
        b01 = 0
        b10 = s2[0]
        b11 = s2[1]
        if corners[0][0] < 0:
            overflow = - corners[0][0]
            b00 = overflow
        if corners[1][0] > bounds[0]:
            overflow = corners[1][0] - bounds[0]
            b10 = s2[0] - overflow
        if corners[0][1] < 0:
            overflow = - corners[0][1]
            b01 = overflow
        if corners[1][1] > bounds[1]:
            overflow = corners[1][1] - bounds[1]
            b11 = s2[1] - overflow

    return corners, ((b00, b01), (b10, b11)) if b else None




def clamp(val, low, high):
    if val < low:
        return low
    if val > high:
        return high
    return val

def tadd(*terms, limit=None):
    if len(terms[0]) == 2:
        out = sum(t[0] for t in terms), sum(t[1] for t in terms)
        if limit:
            out = clamp(out[0], 0, limit[0]-1), clamp(out[1], 0, limit[1]-1)
        return out
    else:
        if limit:
            return tuple(clamp(sum(t[i] for t in terms), 0, limit[i]-1) for i in len(terms[0]))
        else:
            return tuple(sum(t[i] for t in terms) for i in len(terms[0]))

def tsub(t1,t2, limit=None):
    out = t1[0] - t2[0], t1[1] - t2[1]
    if limit:
        out = clamp(out[0], 0, limit[0]-1), clamp(out[1], 0, limit[1]-1)
    return out

def slice(arena, c):
    return arena[c[0][0]:c[1][0], c[0][1]:c[1][1]]

# Set up a flat field
land = mkarena(100, level)

# Precompute stuff for this ball shape
ball, bmask = mkball(r, level=level)
dist, pushx, pushy , px2, py2 = centerdistpush(ball.shape)
prender(dist, pushx, pushy, px2, py2)

# Initial ball drop in the center. Just delete the extra sand.
pos = (land.shape[0]//2, land.shape[1]//2)
c, _ = find_corners(pos, ball.shape, land.shape)
target = slice(land,c) 
ix = target > ball
target[ix] = ball[ix]

# Find the sand that's in the way if we move the ball by one pixel
path = [(1, 1)] * 12  + [(-1, 0)] * 18 + [(0, -1)] * 18
prender(land)
for delta in path:
    pos2 = tadd(pos, delta)
    c, b = find_corners(pos2, ball.shape, land.shape)
    #print(pos, c, b)
    local = slice(land, c)
    lball = ball
    lmask = bmask
    if b:
        lball = slice(ball, b)
        lmask = slice(bmask, b)

    # disturbed sand
    def mkdisturbance(local, lball, lmask):
        pushed = (local > lball) * lmask
        ix = np.nonzero(pushed)
        disturbance = (local - lball) * lmask
        return disturbance, ix

    disturbance, ix = mkdisturbance(local, lball, lmask)
    last = 0
    while disturbance is None or (len(ix[0]) > 0 and np.count_nonzero(disturbance[ix]) != last):
        cc = (local.shape[0]//2, local.shape[1]//2)
        indicies = list(zip(*ix))
        indicies.sort(key=lambda i: dist[i], reverse=True)
        #print(indicies, ix)
        ix2 = tuple(map(lambda *a: tuple(a), *indicies))
        last = np.count_nonzero(disturbance[ix])
        #print(last)

        def push(pos, poslocal, land, local, i, grains, needs_settling, mult=0):
            """
            pos: offset vector of local in land space
            land: global space
            local: local space
            i: cell to push grains off of in local space
            poslocal: pos in local space
            grains: number o grains to push
            needs_settling: output list of new locations that need slope checking.
            """
            print(local.shape, poslocal)
            pushdelta = pushdir_array(local.shape, poslocal)
            #prender(pushdelta[0], pushdelta[1])
            for _ in range(grains):
                offset = 0, 0
                while offset == (0, 0):
                    rx = 1 - (random.random() < CRITICAL_SLOPE)
                    ry = 1 - (random.random() < CRITICAL_SLOPE)
                    offset = (
                            int(random.triangular(0, mult, 1) + 1) * pushdelta[0][i] * rx,
                            int(random.triangular(0, mult, 1) + 1) * pushdelta[1][i] * ry
                            )
                    #print(i, offset, pushdelta[0][i], pushdelta[1][i], [rx, ry])
                to = tadd(pos, i, offset, limit=land.shape) 

                # building a pile. Mark it for avalanche checking later
                if (land[to] - local[i]) > 1 and (not needs_settling or needs_settling[-1] != to):
                    if to[0] == 99:
                        print("corner!", i, to, local.shape, land.shape, land[to])
                    needs_settling.append(to)
                local[i] -= 1
                #print(i, "+", offset, "->",to, local.shape)
                land[to] += 1

        # For all of the sand that's getting pushed, push it.
        needs_settling = []
        for i in indicies:
            grains = disturbance[i]
            push(c[0], (c[0][0] - i[0], c[0][1] - i[1]), land, local, i, grains, needs_settling)

        # TODO actually settle sand to below angle of repose rather than randomly getting it close
        needs_settling.reverse()
        k = 0
        last_needs_settling = []
        while needs_settling and set(last_needs_settling) != set(needs_settling):
            k += 1
            #print('settling round', k, needs_settling)
            peaks = needs_settling
            needs_settling = []
            for i in peaks:
                pushdelta = pushdir(pos2, i)
                to = tadd(i, pushdelta, limit=land.shape)
                d = dist2d(i, to)
                #print ("settle", i, "->", to, ":", land[i], land[to])
                if abs(land[i] - land[to]) != 1 and (land[i] - land[to]) / d > CRITICAL_SLOPE:
                    grains = max(0, int(land[i] - land[to] - CRITICAL_SLOPE * d))
                    for k in range(grains):
                        push((0, 0), pos2, land, land, i, 1, needs_settling, mult=k*CRITICAL_SLOPE*CRITICAL_SLOPE)




        delta_display = np.zeros(local.shape, dtype=np.uint8)
        delta_display[ix] = disturbance[ix]
        focused_heightmap = np.zeros(local.shape, dtype=np.uint8)
        focused_heightmap[ix] = local[ix]
        prender(local, delta_display, focused_heightmap)

        disturbance, ix = mkdisturbance(local, lball, lmask)

    pos=pos2

prender(land)









