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

### 0. **Codey, god I hate codey**
- I get it you wanna show off your skills that is why you write 1000 line of macaroni code yes I am talking to you (Sirmadeira), NO CODEY, make simple code.
- Code is supposed to  be simple, avoid types everywhere if there is no reason for it avoid fancy stuff. Split the taks into parts and follow bevy + lightyear general struct usage.

### 1. **No Empty Struct Fields**
- **Policy**: Empty struct fields are strictly prohibited. Every field in a struct must have a meaningful and valid value.  

### 2. **Code Consistency**
- Follow consistent naming conventions across all plugins. Shared logic in the `shared` plugin must be decoupled and reusable by both `server` and `client`.
- Use proper comments and documentation where required. Undocumented "magic structs" or placeholder values are forbidden.
- OH YEAH, if overlapping plugin divisions like player in server, and player in client add the keyword client and server.

### 3. **Plugins EVERYWHERE**
- We want our code to be very modular, similar to the game Roblox, so we can easily turn on and off features via disabling plugins.

### 4. **Resources pre initialized**
- We rarely want a resource to not be initialized right when we are booting up game. IF you gonna make a global variable use it and abuse it right in the beginning. Perhaps we can clear her memory on a later moment. Or perhaps we will always will want her ready and available

### 5. **Abuse egui**
- It is easy dont worry about it, is also very easy to despawn eguis in general. You made an interesting mechanic make an egui for it.
- EXAMPLE: Made character customizer - Make egui for it. Then we worry about doing a cool UI for it, lobby? Make an egui for it, cry yourself to sleep? Well EGUI is the answer

## 6. **Marker on marker components**
- Marker component are very usefull when you have  multiple similar entities, I get it.
- But hell man name them Marker something so everybody knows it is a marker, sometime we be tripping you get it dont you? thanks!

## 7. **Want assets? Asset loader is for you**
- So is very simple, just check load_assets file in client, wanna load assets do it via bevy asset loader, he can make maps vectors whatever you want for you.
- He also makes loading states which are extremely usefull. He avoid so much codey i love him.

## 8. **Control is everything**
- You see I like crates as much as the next guy, but never ever use low code option that take out our ability to do something.
- Example: We have character controller in rapier right? Well that thing is good if this was an alpha which it isnt we must be able to CONTROL everything.
- Other example: Lightyear makes spawning and despawning character session based, of course this avoids usage of code. But it takes our ability to control AFK status.



---
