<!-- 前端架构梳理 -->
```mermaid
    

```

```txt
const task = {
  id: 1,
  title: "'test'",
  compele: false,
  details: "'test'",
  task_order: 1,
  priority: 1,
};


// insert_task
invoke("insert_task", { task })
  .then((result) => {
    console.log("success：", result);
  })
  .catch((err) => {
    console.error("error：", err);
  });

  invoke("query_task", {
    whereClause: "id = ?",
    params: { id: 1 },
    task: { id: 1, title: "xxx", compele: false, details: "test", task_order: 1, priority: 1 }
  })
  .then((result) => {
    console.log("success：", result);
  })
  .catch((err) => {
    console.error("error：", err);
  });

```