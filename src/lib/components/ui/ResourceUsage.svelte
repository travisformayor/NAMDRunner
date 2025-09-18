<script lang="ts">
  import Progress from "./Progress.svelte";
  import Card from "./Card.svelte";

  export let title: string;
  export let usage: number;
  export let total: string;
  export let unit: string = "";
  export let color: string = "blue";

  $: percentage = Math.min(Math.max(usage, 0), 100);
  $: colorClass = getColorClass(color);

  function getColorClass(color: string) {
    const colors: Record<string, string> = {
      blue: "bg-blue-600",
      green: "bg-green-600",
      yellow: "bg-yellow-600",
      red: "bg-red-600",
      purple: "bg-purple-600"
    };
    return colors[color] || "bg-blue-600";
  }
</script>

<Card>
  <div class="p-4">
    <div class="flex items-center justify-between mb-2">
      <h4 class="text-sm font-medium">{title}</h4>
      <span class="text-xs text-muted-foreground">{usage.toFixed(1)}%</span>
    </div>

    <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 mb-2">
      <div
        class="h-2.5 rounded-full transition-all duration-300 ease-in-out {colorClass}"
        style="width: {percentage}%"
      ></div>
    </div>

    <div class="text-xs text-muted-foreground">
      {total} {unit}
    </div>
  </div>
</Card>