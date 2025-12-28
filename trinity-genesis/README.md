# Trinity AI OS

> **"The Tool for New AI Educational Environments."**

## Vision: Constructivism over Rote Memorization

Trinity is a **Pure-Rust, Local-First AI Operating System** designed for public school classrooms. It is not just an educational game; it is the **Authoring Tool** that empowers Instructional Designers and Teachers to build **Constructivist Video Game Experiences** (like "Iron Road" or "Ask Pete") without needing a team of software engineers.

We believe that:

1. **Data Sovereignty is Non-Negotiable**: Student data must never leave the classroom. Trinity runs 100% locally on standard hardware (targeting AMD Strix Halo / High-end Consumer PCs).
2. **Learning is Kinetic**: Education is not about "downloading" facts (rote memorization); it is about "constructing" schemas through action, play, and struggle (Constructivism).
3. **Tools Shape Minds**: By giving teachers a "Video Game UI" to design curriculum, we change the nature of what is taught from "static content" to "dynamic systems."

## The Architecture

Trinity is built on a unified, high-performance Rust stack to ensure stability, safety, and speed.

- **The Mind (Backend)**: `trinity-brain` (Axum + Llama-cpp-2).
  - A local AI Orchestrator that acts as a "Socratic Mirror," guiding students and helping teachers design content.
- **The Face (Frontend)**: `trinity-client` (Bevy Engine).
  - A Native/WASM Video Game Interface. It serves as both the **Classroom Dashboard** for students and the **"Level Editor"** for teachers.
- **The Logic (Physics)**: "Coal & Steam" Economy.
  - An internal simulation of Cognitive Load Theory, modeled as a physics engine.

## Usage

Trinity is designed to be deployed on a single classroom computer (The "Server") which students connect to, or run continuously on personal devices.

### For Teachers (Instructional Designers)

Use Trinity's **Agentic UI** to describe a lesson plan. The AI Agent assists in scaffolding the "Game Level" (Curriculum), ensuring it meets pedagogical standards (ZPD, Cognitive Load constraints).

### For Students

Log in to a gamified dashboard where "Vocabulary" is inventory and "Learning" is movement.

## License

**FOSS (Free and Open Source Software)**. Built for the public good, forever free for public education.
