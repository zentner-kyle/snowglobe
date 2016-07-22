As usual, let's try to write tic-tac-toe:
```
player = X | O
winner = X | O | _
board = [0..3][0..3] => X | O | _

init:
  board[*x][*y]:
    board[x][y] = _
  player = X
  winner = _

winner == _ and board[*x][*y] == _ and control(player):
  board[x][y] = player
  player == X:
    player = O
  player == O:
    player = X

board[*x][*y] != _:
  (board[x][y] == board[x + 1][y] and board[x][y] == board[x + 2][y]) or
  (board[x][y] == board[x][y + 1] and board[x][y] == board[x][y + 2]) or
  (board[x][y] == board[x + 1][y + 1] and board[x][y] == board[x + 2][y + 2]) or
  (board[x][y] == board[x + 1][y - 1] and board[x][y] == board[x + 2][y - 2]):
    winner = board[x][y]
```

```
(variables
  (player (0 1))
  (winner (0 1 2))
  (board '(((0 1 2) (0 1 2) (0 1 2))
           ((0 1 2) (0 1 2) (0 1 2))
           ((0 1 2) (0 1 2) (0 1 2)))))

(variables
  (player 0)
  (winner 2)
  (board '((2 2 2)
           (2 2 2)
           (2 2 2))))

(if (== winner 2)
  (for (x row) board
    (for (y p) row
      (if (== p) 2)
        (ifdo (control player)
          (= (elt y (elt x board)) player)
          (if (== player 1)
            (= player 2))
          (if (== player 2)
            (= player 1))))))

(for (x row) board
  (for (y p) row
    (if (not (== p 2))
      (if (or (and (== p (elt y (elt (+1 x) board))) (== p (elt y (elt (+1 (+1 x)) board))))
              (and (== p (elt (+1 y) (elt x board))) (== p (elt (+1 (+1 y)) (elt x board))))
              (and (== p (elt (+1 y) (elt (+1 x) board))) (== p (elt (+1 (+1 y)) (elt (+1 (+1 x)) board))))
              (and (== p (elt (-1 y) (elt (+1 x) board))) (== p (elt (-1 (-1 y)) (elt (+1 (+1 x)) board)))))
          (= winner p)))))
```

Connect 4:
```
player = X | O
winner = X | O | _
board = [0..6][0..7] => X | O | _

init:
  board[*x][*y]:
    board[x][y] = _
  player = X
  winner = _

winner == _ and board[*x][*y] == _ and board[x][y - 1] != _ and control(player):
  board[x][y] = player
  player == X:
    player = O
  player == O:
    player = X

board[*x][*y] != _:
  (board[x][y] == board[x + 1][y] and board[x][y] == board[x + 2][y] and board[x][y] == board[x + 3][y]) or
  (board[x][y] == board[x][y + 1] and board[x][y] == board[x][y + 2] and board[x][y] == board[x][y + 3]) or
  (board[x][y] == board[x + 1][y + 1] and board[x][y] == board[x + 2][y + 2] and board[x][y] == board[x + 3][y + 3]) or
  (board[x][y] == board[x + 1][y - 1] and board[x][y] == board[x + 2][y - 2] and board[x][y] == board[x + 3][y - 3]):
    winner = board[x][y]
```

Toot-n-otto:
```
player = X | O
winner = X | O | _
board = [0..6][0..6] => X | O | _

init:
  board[*x][*y]:
    board[x][y] = _
  player = X
  winner = _

winner == _ and board[*x][*y] == _ and board[x][y - 1] != _ and control(player):
  board[x][y] = player
  player == X:
    player = O
  player == O:
    player = X

board[*x][*y] != _:
  (board[x][y] == board[x + 1][y] and board[x][y] == board[x + 2][y] and board[x][y] == board[x + 3][y]) or
  (board[x][y] == board[x][y + 1] and board[x][y] == board[x][y + 2] and board[x][y] == board[x][y + 3]) or
  (board[x][y] == board[x + 1][y + 1] and board[x][y] == board[x + 2][y + 2] and board[x][y] == board[x + 3][y + 3]) or
  (board[x][y] == board[x + 1][y - 1] and board[x][y] == board[x + 2][y - 2] and board[x][y] == board[x + 3][y - 3]):
    winner = board[x][y]
```

11 variables, either 0, 1, 2
==
and
or
not
+1
-1
index
assign
foreach
