# Technical README: Infrastructure Guidelines

Welcome to the technical README for our project infrastructure. This document outlines the core guidelines and principles for managing and contributing to the project. Adhering to these rules ensures consistency, maintainability, and overall project quality.

---

## Project Structure

The project is divided into three core sub-plugins:

1. **`server`**  
   Responsible for backend logic, data handling, and communication with the client. This plugin serves as the backbone for the project's infrastructure.

2. **`client`**  
   Handles the frontend logic and the user interface. This plugin represents the player-facing side of the system.

3. **`shared`**  
   Contains shared resources, utilities, and logic that are reused between `server` and `client`. It ensures DRY (Don't Repeat Yourself) principles are followed and provides a centralized location for common functionality.

---

## Key Guidelines

### 1. **No Empty Struct Fields**
- **Policy**: Empty struct fields are strictly prohibited. Every field in a struct must have a meaningful and valid value.  
- **Example**: Avoid accessing fields like `struct.field.0` unless explicitly designed and documented.  
- **Action**: If you encounter a struct field being accessed via `0` (or other unintended empty behaviors), refactor it immediately. If it cannot be justified, **remove it**.

### 2. **Code Consistency**
- Follow consistent naming conventions across all plugins. Shared logic in the `shared` plugin must be decoupled and reusable by both `server` and `client`.
- Use proper comments and documentation where required. Undocumented "magic numbers" or placeholder values are forbidden.

### 3. **Plugins EVERYWHERE**
- We want our code to be very modular, similar to the game Roblox, so we can easily turn on and off features via disabling plugins.

### 4. **Resources pre initialized**
- We rarely want a resource to not be initialized right when we are booting up game. IF you gonna make a global variable use it and abuse it right in the beginning. Perhaps we can clear her memory on a later moment. Or perhaps we will always will want here ready and available

### 5. **Abuse egui**
- It is easy dont worry about it, is also very easy to despawn eguis in general. You made an interesting mechanic make an egui for it.
- EXAMPLE: Made character customizer - Make egui for it. Then we worry about doing a cool UI.
---
