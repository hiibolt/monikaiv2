# MonikaiV2
A user-assistant model prompter with powerful memory and context features that create an incredibly human experience over longer periods of time. Based on the [MemoryBank whitepaper](https://arxiv.org/pdf/2305.10250.pdf) by Wanjun Zhong, Lianghong Guo1, Qiqi Gao, He Ye, and Yanlin Wang.

MonikaiV2 instances feature a Short-Term Memory (STM) and Long-Term Memory (LTM), while also mimicing conscious/subconscious retrieval cues, retrieval failure, and selective forgetting.

## Prompting Methods
### Read-Eval-Print-Loop (REPL)
![image](https://github.com/hiibolt/monikaiv2/assets/91273156/72a8dfd9-d606-473b-ae1a-bd2b4aec47d2)

 A Read-Eval-Print Loop (REPL) communication interface for Monikai.
 The least convoluted method of communication, very straightforward. 
It's worth noting that conversations can be had seamlessly between the online client and the REPL, as demonstrated in the below photo.

 Anything which is not a command is forwarded as a prompt to the Monikai.

 #### Commands:
- **wipe**: Clears the Monikai's memories and recent conversation, preserves the description.
- **save**: Writes the Monikai in memory to 'monikai.json'.
- **end**: Manually marks the current conversation as completed and encodes it to memory.
- **log**: Prints the Monikai in memory to stdout.
- **get**: Takes another line as input, and prints the memory most similar in cosine.

### Web Client

When starting the repo, a graphical web client is hosted on port **3000**.

This contains an additional interface layer, allowing the Monikai to express a preset range of emotions! This can create a more human-like interaction, and puts a face to text.

All assets can be customized by replacing the files in **./public/assets**. Ensure that you modify either the import code in **./public/index.html** or mimic the original file names.

*Note: A given emotion must have two files to be properly rendered: "EMOTION.png" and "EMOTIONSPEAKING.png". If you don't want a speaking version, simply duplicate and rename EMOTION.png.*


![image](https://github.com/hiibolt/monikaiv2/assets/91273156/acd1d435-e91b-4bf2-8ad7-51df6c5af850)
![image](https://github.com/hiibolt/monikaiv2/assets/91273156/3cb51a7f-3888-4561-8213-a6f2d6b94fd8)

## Automatic Memory Pruning
MonikaiV2's memory pruning system draws insight from the [Trace Decay Theory of Forgetting](https://practicalpie.com/theories-of-forgetting/) and the [Ebbinghaus Forgetting Curve](https://practicalpie.com/theories-of-forgetting/).

During testing, I found that this created the most 'human' experience and lead to the least hallucination.

Memories are not pruned until 7 days have passed, which is considered the average time a human remembers a conversation, after which they are subject to forgetting.

**The following equation was used to model 'forgetting':**
![image](https://github.com/hiibolt/monikaiv2/assets/91273156/5e6fd232-1074-4238-8c87-35e2cc2a6f80)

Where **d** is the number of days since the memory was formed and **r** is the number of times the memory has been retrived. Memories with a score over 1 are forgotten.

The model retrieves a memory either  subconsciously with a 'retrieval cue', or consciously if the Monikai determines that it needs more context.

### For example:
The user prompts the Monikai "Hey, what was the baking cookbook you recommended me?"
- **Subconsciously**: 
    1. Subconscious memory search with key phrase "baking cooking"
    2. A memory where the user talked about lunch, where the user ate a baked pretzel
- **Consciously**: 
    1. The Monikai determines they need more information
    2. Conscious memory search with key phrase "cookbook recommendation"
    3. A memory where the Monikai recommended *The College Cookbook*.
## Autosave / Auto-Encoding
Monikai will save automatically every 5 senconds, and a conversation is considered 'over' after 5 minutes of inactivity. When a conversation is 'over', it will automatically self-encode into LTM.

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

3. (Optional) Customize the character description field in **monikai/data/monikai.json**.
3. Start the Monikai!

    ```cargo run```

#### Credits:
- "Nastya Sprite Pack" by u/uzikovskikh ([link](https://www.reddit.com/r/DDLC/comments/15qcmp9/content_pack_release_nastya_by_uvitkovskikh_and/))
- "MemoryBank: Enhancing Large Language Models
with Long-Term Memory" - ([link](https://arxiv.org/pdf/2305.10250.pdf))
