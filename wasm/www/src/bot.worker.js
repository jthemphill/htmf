const ctx = self;

import("./Bot")
  .catch(e => console.error("Error importing `Bot.ts`:", e))
  .then((module) => {
    const bot = new module.default(ctx.postMessage);
    ctx.onmessage = (event) => {
      bot.onMessage(event.data);
    };
    bot.init();
  });
