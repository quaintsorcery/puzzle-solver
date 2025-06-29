# Puzzle Solver
A graphical app for complex multi-step text transformation.

Initially built for solving puzzles with various ciphers and other encryption techniques.

# How to use
1. Create an **Input** node, that's where you put your initial text
2. Link your **Input** to a **Transformer** node
3. The output of a **Transfromer** node can be of type **Text**, **List**, or **Error**, as signified by the wire color, and can be piped into further **Transformer** nodes

# Transformers
- **Split** - turns **Texts** into **Lists** of Texts split by *Pattern*
- **Join** - turns **Lists** into a **Text** with a *Separator*
- **Find** - turns **Texts** into **Lists** of found *Patterns*
- **Replace** - replaces a *Patern* with a *Replacer* in **Texts**
- **Slice** - cuts **Texts** into slices *from* one index *to* another
- **Encode** (may output **Errors**):
    - **Base64**: Base64-encodes **Texts**
    - **Base64 URL Safe**: Base64-encodes **Texts** (URL safe)
    - **URL**: URL-encodes **Texts**
- **Decode** (may output **Errors**):
    - **Base64**: Base64-decodes **Texts**
    - **Base64 URL Safe**: Base64-decodes **Texts** (URL safe)
    - **URL**: URL-decodes **Texts**
- **Uppercase** - converts **Texts** to uppercase
- **Lowercase** - converts **Texts** to lowercase

---

Built using Rust and [egui-snarl](https://github.com/zakarumych/egui-snarl)

