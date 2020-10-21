# Word Ladders

Word ladders were invented by Lewis Carroll, the author of Alice in Wonderland, in 1878. In
a word ladder puzzle, you try to go from the starting word to the ending word, by changing
a word into another word by altering a single letter at each step. Each word in the ladder
must be a valid English word, and must have the same length. For example, to turn stone
into money, one possible ladder is shown here:

stone atone alone clone clons coons conns cones coney money

There are several ways to solve the problem of building a word ladder between given
starting and ending words. One simple method involves stacks and queues.

The algorithm works as follows:
Get the starting word and search through the wordlist to find all words that are one
letter different. Create stacks for each of these words, containing the starting word
(pushed first) and the word that is one letter different. Enqueue each of these stacks
into a queue. This will create a queue of stacks! Then dequeue the first item (which
is a stack) from the queue, look at its top word and compare it with the ending word.
If they are equal, you are done – this stack contains the ladder. (Return it!)
Otherwise, you find all of the words one letter different from it. For each of these
new words, create a deep copy of the stack (use copy.deepcopy()) and push each
word onto the stack. Then enqueue those stacks to the queue. And so on. You
terminate the process when you reach the ending word or the queue is empty.
As you go, you have to keep track of used words, otherwise an infinite loop occurs.
Also, make sure to check that the start word and end word aren’t the same!

## Example
The starting word is smart. Find all the words one letter different from smart, push them
into different stacks and store stacks in the queue. This table represents a queue of stacks.

| scart  | start | swart | smalt | smarm |
| ---    | ---   | ---   | ---   | ---   |
| smart  | smart | smart | smart | smart |

Now dequeue the front and find all words one letter different from the top word scart. This
will spawn seven stacks:

| scant  | scatt | scare | scarf | scarp | scars | scary |
|   ---  |  ---  |  ---  |  ---  |  ---  |  ---  |  ---  |
| scart  | scart | scart | scart | scart | scart | scart |
| smart  | smart | smart | smart | smart | smart | smart |

which we enqueue to the queue. The queue size is now 11. Again, dequeue the front and
find all words one letter different from the top word start. This will spawn four stacks:

| sturt | stare | stark | stars |
| ---   | ---   | ---   | ---   |
| start | start | start | start |
| smart | smart | smart | smart |

Add them to the queue. The queue size is now 14. Repeat the procedure until you find the
ending word or such a word ladder does not exist. Make sure you do not run into an infinite
loop!

##Usage

###Not Optimized: `cargo run -- <MODE> <THREADS>`
###Optimized: `cargo run --release -- <MODE> <THREADS>`
MODE: 
- -g (for building the complete graph in parallel)
- -d (for building neighborhoods dynamically)



