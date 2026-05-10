export default function App() {
  return (
    <div className="container">
      <h1 className="header" id="unique-header">Advanced JSX</h1>
      
      {/* Tag specific */}
      <div className="text-box">JSX Div</div>
      <input className="text-box" />

      {/* Pseudo */}
      <button className="interactive">Hover Me</button>
      
      {/* Scale */}
      <div className="scaled">Box</div>
    </div>
  );
}
