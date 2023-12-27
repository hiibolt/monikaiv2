# MonikaiV2
A user-assistant model prompter with powerful context and memory features that replicate a very human experience over long time periods.

MonikaiV2 instances feature a Short-Term Memory (STM), Long-Term Memory (LTM), and retrieval/encoding features. It also mimics conscious/subconscious retriveal cues, retrieval failure, and selective forgetting.

## Methods of communication
### Read-Eval-Print-Loop (REPL)
 A Read-Eval-Print Loop (REPL) communication interface for Monikai.
 The least convoluted method of communication, very straightforward.

 Anything not a command forwards prompts the Monikai.

 #### Commands:
- **wipe**: Clear the Monikai's memories and recent conversation, preserves the description.
- **save**: Writes the Monikai in memory to 'monikai.json'.
- **end**: Manually marks the current conversation as completed and encodes it as a memory.
- **log**: Prints the Monikai in memory to stdout.
- **get**: Takes another line as input, and prints the memory most similar in cosine.

### Web Client
When starting the repo, a graphical web client is hosted on port **3000**.

There is an additional interface layer, allowing the Monikai to express a preset range of emotions! This can add another level of humanization.


All assets can be customized by replacing the files in **./public/assets**. Ensure that you modify either the import code in **./public/index.html** or mimic the names.

*Note: A given emotion must have two files to be properly rendered: "EMOTION.png" and "EMOTIONSPEAKING.png". If you don't want a speaking version, simply duplicate and rename EMOTION.png.*

## Automatic Memory Pruning
MonikaiV2's memory pruning system draws insight from the Trace Decay Theory of Forgetting and the Ebbinghaus Forgetting Curve.

During testing, I found that this created the most 'human' experience and lead to the least hallucination.

Memories are not pruned until 7 days have passed, which is considered the average time a human remembers a conversation, after which they are subject.

**The following equation was used to model 'forgetting':**

Where **d** is the number of days since the memory was formed and **r** is the number of times the memory has been retrived.

The model retrives a memory either subconciously with a 'retrieval cue', or conciously if it determines that it needs more context.

### For example:
The user prompts the Monikai "Hey, what was the baking cookbook you recommended me?"
- **Subconciously**: 
    1. Subconcious memory search with key phrase "baking cooking"
    2. A memory where the user talked about lunch, where the user ate a baked pretzel
- **Conciously**: 
    1. The Monikai determines they need more information
    2. Concious memory search with key phrase "cookbook recommendation"
    3. A memory where the Monikai recommended *The College Cookbook*.
## Autosave / Auto-Encoding
Monikai will save automatically every 5 sencods, and a conversation is considered 'over' after 5 minutes of inactivity. When a conversation is 'over', it will automatically self-encode into LTM.

# Usage
## Prerequisites:
- [Git](https://git-scm.com/downloads)
- [Rust](https://www.rust-lang.org/tools/install)
- [An OpenAI API key](https://openai.com/product/)
## Setup
1. Start by cloning the repo:

    ```git clone https://github.com/hiibolt/monikaiv2.git```

2. Set environment key **OPENAI_API_KEY** to your OpenAI API key.

    ```export OPENAI_API_KEY="YOUR KEY HERE"``` *(method varies)*

3. (Optional) Edit the description **monikai/data/monikai.json** to meet your needs for the prompt.
3. Start the Monikai!

    ```cargo run```

#### Credits:
- "Nastya Sprite Pack" by u/uzikovskikh ([link](https://www.reddit.com/r/DDLC/comments/15qcmp9/content_pack_release_nastya_by_uvitkovskikh_and/))